use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use cartile_format::{
    CartileMap, Grid, GridType, HeightMode, HexOrientation, Layer, MapObject, ObjectLayer, Point,
    Projection, ProjectionType, Properties, Property, PropertyType, Shape, Stagger, StaggerAxis,
    StaggerIndex, TileId, TileLayer, Tileset, TilesetEntry, TilesetFile, TilesetRef, Topology,
};

use super::types::{
    TiledLayer, TiledMap, TiledObject, TiledProperty, TiledTileset, TiledTilesetEntry,
};

const TILED_GID_MASK: u32 = 0x0FFF_FFFF;
const HEX_ROTATION_BIT: u32 = 0x1000_0000;
const CARTILE_FLAG_MASK: u32 = 0xE000_0000;

/// Convert a Tiled GID (with 4 flag bits) to a cartile raw TileId value (3 flag bits).
/// Returns (cartile_raw, warnings).
pub fn convert_gid(tiled_raw: u32) -> (u32, Vec<String>) {
    if tiled_raw == 0 {
        return (0, vec![]);
    }
    let mut warnings = vec![];
    let tiled_flags = tiled_raw & 0xF000_0000;
    let tiled_gid = tiled_raw & TILED_GID_MASK;

    if tiled_flags & HEX_ROTATION_BIT != 0 {
        warnings.push("tile has hex rotation flag (bit 28) — cleared".to_string());
    }

    let cartile_flags = tiled_flags & CARTILE_FLAG_MASK;
    (cartile_flags | tiled_gid, warnings)
}

/// Convert a Tiled color string to cartile format.
/// Tiled: `#AARRGGBB` or `#RRGGBB`
/// Cartile: `#RRGGBBAA`
pub fn convert_color(tiled_color: &str) -> String {
    let hex = tiled_color.trim_start_matches('#');
    match hex.len() {
        8 => {
            let aa = &hex[0..2];
            let rrggbb = &hex[2..8];
            format!("#{rrggbb}{aa}")
        }
        6 => format!("#{hex}FF"),
        _ => tiled_color.to_string(),
    }
}

/// Convert a single Tiled property to cartile format.
/// Returns (name, Property, warnings). If the property type is unsupported,
/// warnings will be non-empty.
pub fn convert_property(tp: &TiledProperty) -> (String, Property, Vec<String>) {
    let mut warnings = vec![];
    let (prop_type, value) = match tp.property_type.as_str() {
        "string" => (PropertyType::String, tp.value.clone()),
        "int" => (PropertyType::Int, tp.value.clone()),
        "float" => (PropertyType::Float, tp.value.clone()),
        "bool" => (PropertyType::Bool, tp.value.clone()),
        "file" => (PropertyType::File, tp.value.clone()),
        "color" => {
            let converted = tp
                .value
                .as_str()
                .map(|s| serde_json::Value::String(convert_color(s)))
                .unwrap_or_else(|| tp.value.clone());
            (PropertyType::Color, converted)
        }
        other => {
            warnings.push(format!(
                "unsupported property type '{other}' for '{}' — skipped",
                tp.name
            ));
            (PropertyType::String, tp.value.clone())
        }
    };
    (
        tp.name.clone(),
        Property {
            property_type: prop_type,
            value,
        },
        warnings,
    )
}

/// Convert a Vec of Tiled properties to a cartile Properties map.
/// Unsupported types are skipped.
pub fn convert_properties(tiled_props: &[TiledProperty]) -> (Properties, Vec<String>) {
    let mut props = Properties::new();
    let mut all_warnings = vec![];
    for tp in tiled_props {
        let (name, prop, warnings) = convert_property(tp);
        let is_unsupported = !warnings.is_empty();
        all_warnings.extend(warnings);
        if !is_unsupported {
            props.insert(name, prop);
        }
    }
    (props, all_warnings)
}

// ---------------------------------------------------------------------------
// Main conversion entry point
// ---------------------------------------------------------------------------

/// Convert a parsed `TiledMap` into a `CartileMap`.
///
/// - `map_name`: used as `CartileMap::name`
/// - `input_dir`: directory of the source .tmj file (for resolving external tileset paths)
/// - `output_dir`: when `Some`, external tilesets are written as `.cartile-ts` files
///   and referenced; when `None`, all tilesets are inlined
///
/// Returns the converted map and a list of non-fatal warnings.
pub fn convert_tiled_map(
    tiled: &TiledMap,
    map_name: &str,
    input_dir: &Path,
    output_dir: Option<&Path>,
) -> anyhow::Result<(CartileMap, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();

    // Reject infinite/chunk-based maps
    if tiled.infinite {
        anyhow::bail!("infinite/chunk-based maps are not supported");
    }

    // Warn if the render order is non-default
    if let Some(ref order) = tiled.renderorder
        && order != "right-down"
    {
        warnings.push(format!(
            "non-default renderorder '{order}' — cartile always uses right-down"
        ));
    }

    let (properties, prop_warnings) = convert_properties(&tiled.properties);
    warnings.extend(prop_warnings);

    let grid = convert_grid(tiled)?;
    let tilesets = convert_tilesets(tiled, input_dir, output_dir, &mut warnings)?;
    let layers = convert_layers(&tiled.layers, &mut warnings);

    let map = CartileMap {
        cartile: "0.1.0".to_string(),
        map_type: "map".to_string(),
        name: map_name.to_string(),
        properties,
        grid,
        tilesets,
        layers,
        extra: HashMap::new(),
    };

    Ok((map, warnings))
}

// ---------------------------------------------------------------------------
// Grid conversion
// ---------------------------------------------------------------------------

fn convert_grid(tiled: &TiledMap) -> anyhow::Result<Grid> {
    let (grid_type, projection_type, stagger, orientation) = match tiled.orientation.as_str() {
        "orthogonal" => (GridType::Square, ProjectionType::Orthogonal, None, None),
        "isometric" => (GridType::Square, ProjectionType::Isometric, None, None),
        "staggered" => {
            let stagger = parse_stagger(tiled)?;
            (
                GridType::Square,
                ProjectionType::Isometric,
                Some(stagger),
                None,
            )
        }
        "hexagonal" => {
            let stagger = parse_stagger(tiled)?;
            let orientation = match tiled.staggeraxis.as_deref() {
                Some("y") => HexOrientation::PointyTop,
                Some("x") => HexOrientation::FlatTop,
                _ => {
                    return Err(anyhow::anyhow!("hexagonal map missing staggeraxis"));
                }
            };
            (
                GridType::Hexagonal,
                ProjectionType::Orthogonal,
                Some(stagger),
                Some(orientation),
            )
        }
        other => {
            return Err(anyhow::anyhow!("unknown orientation: '{other}'"));
        }
    };

    Ok(Grid {
        grid_type,
        width: tiled.width,
        height: tiled.height,
        tile_width: tiled.tilewidth,
        tile_height: tiled.tileheight,
        orientation,
        stagger,
        topology: Topology::Bounded,
        projection: Projection {
            projection_type,
            angle: None,
            extra: HashMap::new(),
        },
        height_mode: HeightMode::None,
        extra: HashMap::new(),
    })
}

fn parse_stagger(tiled: &TiledMap) -> anyhow::Result<Stagger> {
    let axis = match tiled.staggeraxis.as_deref() {
        Some("x") => StaggerAxis::X,
        Some("y") => StaggerAxis::Y,
        _ => {
            return Err(anyhow::anyhow!(
                "staggered/hexagonal map missing staggeraxis"
            ));
        }
    };
    let index = match tiled.staggerindex.as_deref() {
        Some("odd") => StaggerIndex::Odd,
        Some("even") => StaggerIndex::Even,
        _ => {
            return Err(anyhow::anyhow!(
                "staggered/hexagonal map missing staggerindex"
            ));
        }
    };
    Ok(Stagger { axis, index })
}

// ---------------------------------------------------------------------------
// Tileset conversion
// ---------------------------------------------------------------------------

fn convert_tilesets(
    tiled: &TiledMap,
    input_dir: &Path,
    output_dir: Option<&Path>,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<TilesetEntry>> {
    let mut result = Vec::new();

    for entry in &tiled.tilesets {
        match entry {
            TiledTilesetEntry::Embedded(ts) => {
                let (cartile_ts, ts_warnings) = convert_tileset(ts);
                warnings.extend(ts_warnings);
                result.push(TilesetEntry::Inline(cartile_ts));
            }
            TiledTilesetEntry::External(ext) => {
                // Load the external .tsj file
                let ts_path = input_dir.join(&ext.source);
                let json = std::fs::read_to_string(&ts_path)
                    .with_context(|| format!("failed to read tileset '{}'", ts_path.display()))?;
                let mut tiled_ts: TiledTileset = serde_json::from_str(&json)
                    .with_context(|| format!("failed to parse tileset '{}'", ts_path.display()))?;
                // External .tsj files don't carry firstgid; inject it here
                tiled_ts.firstgid = ext.firstgid;
                let (cartile_ts, ts_warnings) = convert_tileset(&tiled_ts);
                warnings.extend(ts_warnings);

                if let Some(dir) = output_dir {
                    // Write as standalone .cartile-ts file and reference it
                    let ts_filename = format!("{}.cartile-ts", cartile_ts.name);
                    let ts_out_path = dir.join(&ts_filename);

                    let ts_file = TilesetFile {
                        cartile: "0.1.0".to_string(),
                        file_type: "tileset".to_string(),
                        tileset: Tileset {
                            // Standalone files don't include first_gid
                            first_gid: 0,
                            ..cartile_ts.clone()
                        },
                    };
                    let file = std::fs::File::create(&ts_out_path).with_context(|| {
                        format!("failed to write tileset file '{}'", ts_out_path.display())
                    })?;
                    let writer = std::io::BufWriter::new(file);
                    serde_json::to_writer_pretty(writer, &ts_file).with_context(|| {
                        format!(
                            "failed to serialize tileset file '{}'",
                            ts_out_path.display()
                        )
                    })?;

                    result.push(TilesetEntry::ExternalRef(TilesetRef {
                        ref_path: format!("./{ts_filename}"),
                        first_gid: cartile_ts.first_gid,
                    }));
                } else {
                    // Inline the tileset
                    result.push(TilesetEntry::Inline(cartile_ts));
                }
            }
        }
    }

    Ok(result)
}

fn convert_tileset(ts: &TiledTileset) -> (Tileset, Vec<String>) {
    let mut warnings = Vec::new();
    let mut tiles = HashMap::new();

    for tile_def in &ts.tiles {
        if !tile_def.properties.is_empty() {
            let (props, prop_warnings) = convert_properties(&tile_def.properties);
            warnings.extend(prop_warnings);
            tiles.insert(
                tile_def.id.to_string(),
                cartile_format::TileData {
                    properties: props,
                    auto_tile: None,
                    extra: HashMap::new(),
                },
            );
        }
    }

    let cartile_ts = Tileset {
        name: ts.name.clone(),
        tile_width: ts.tilewidth,
        tile_height: ts.tileheight,
        image: ts.image.clone(),
        image_width: ts.imagewidth,
        image_height: ts.imageheight,
        columns: ts.columns,
        tile_count: ts.tilecount,
        margin: ts.margin,
        spacing: ts.spacing,
        first_gid: ts.firstgid,
        tiles,
        extra: HashMap::new(),
    };

    (cartile_ts, warnings)
}

// ---------------------------------------------------------------------------
// Layer conversion
// ---------------------------------------------------------------------------

fn convert_layers(tiled_layers: &[TiledLayer], warnings: &mut Vec<String>) -> Vec<Layer> {
    let mut result = Vec::new();
    for layer in tiled_layers {
        convert_layer(layer, warnings, &mut result);
    }
    result
}

fn convert_layer(tiled_layer: &TiledLayer, warnings: &mut Vec<String>, out: &mut Vec<Layer>) {
    match tiled_layer {
        TiledLayer::TileLayer(tl) => {
            let mut data = Vec::with_capacity(tl.data.len());
            for &raw in &tl.data {
                let (cartile_raw, gid_warnings) = convert_gid(raw);
                warnings.extend(gid_warnings);
                data.push(TileId::from_raw(cartile_raw));
            }
            let (properties, prop_warnings) = convert_properties(&tl.properties);
            warnings.extend(prop_warnings);
            out.push(Layer::Tile(TileLayer {
                name: tl.name.clone(),
                visible: tl.visible,
                opacity: tl.opacity,
                elevation: 0,
                encoding: "dense".to_string(),
                data,
                properties,
            }));
        }
        TiledLayer::ObjectGroup(ol) => {
            let mut objects = Vec::new();
            for obj in &ol.objects {
                // None means skipped (gid/text/template)
                if let Some(map_obj) = convert_object(obj, warnings) {
                    objects.push(map_obj);
                }
            }
            let (properties, prop_warnings) = convert_properties(&ol.properties);
            warnings.extend(prop_warnings);
            out.push(Layer::Object(ObjectLayer {
                name: ol.name.clone(),
                visible: ol.visible,
                opacity: ol.opacity,
                objects,
                properties,
            }));
        }
        TiledLayer::Group(g) => {
            // Flatten group layers into the parent level
            for child in &g.layers {
                convert_layer(child, warnings, out);
            }
        }
        TiledLayer::ImageLayer(img) => {
            warnings.push(format!(
                "image layer '{}' is not supported — skipped",
                img.name
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Object conversion
// ---------------------------------------------------------------------------

fn convert_object(obj: &TiledObject, warnings: &mut Vec<String>) -> Option<MapObject> {
    // Skip tile objects, text objects, and template instances
    if obj.gid.is_some() {
        warnings.push(format!(
            "object {} '{}' is a tile object — skipped",
            obj.id, obj.name
        ));
        return None;
    }
    if obj.text.is_some() {
        warnings.push(format!(
            "object {} '{}' is a text object — skipped",
            obj.id, obj.name
        ));
        return None;
    }
    if obj.template.is_some() {
        warnings.push(format!(
            "object {} '{}' uses a template — skipped",
            obj.id, obj.name
        ));
        return None;
    }

    let (shape, points) = detect_shape(obj);

    let (properties, prop_warnings) = convert_properties(&obj.properties);
    warnings.extend(prop_warnings);

    Some(MapObject {
        id: obj.id,
        name: obj.name.clone(),
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        shape,
        rotation: obj.rotation,
        points,
        properties,
        extra: HashMap::new(),
    })
}

fn detect_shape(obj: &TiledObject) -> (Shape, Option<Vec<Point>>) {
    if obj.point {
        return (Shape::Point, None);
    }
    if obj.ellipse {
        return (Shape::Ellipse, None);
    }
    if let Some(polygon) = &obj.polygon {
        let pts = polygon.iter().map(|p| Point { x: p.x, y: p.y }).collect();
        return (Shape::Polygon, Some(pts));
    }
    if let Some(polyline) = &obj.polyline {
        let pts = polyline.iter().map(|p| Point { x: p.x, y: p.y }).collect();
        return (Shape::Polyline, Some(pts));
    }
    // Default: axis-aligned rectangle
    (Shape::Rect, None)
}
