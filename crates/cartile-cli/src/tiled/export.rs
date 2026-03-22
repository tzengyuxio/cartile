use cartile_format::{
    CartileMap, GridType, Layer, MapObject, ObjectLayer, ProjectionType, Property, PropertyType,
    Shape, StaggerAxis, TileLayer, Tileset, TilesetEntry,
};

use super::types::{
    TiledLayer, TiledMap, TiledObject, TiledObjectLayer, TiledPoint, TiledProperty, TiledTileLayer,
    TiledTileset, TiledTilesetEntry,
};

/// Reverse a cartile color (`#RRGGBBAA`) to Tiled's `#AARRGGBB`.
fn export_color(cartile_color: &str) -> String {
    let hex = cartile_color.trim_start_matches('#');
    match hex.len() {
        8 => {
            let rrggbb = &hex[0..6];
            let aa = &hex[6..8];
            format!("#{aa}{rrggbb}")
        }
        6 => format!("#{hex}"),
        _ => cartile_color.to_string(),
    }
}

/// Convert a cartile Property back to a TiledProperty.
fn export_property(name: &str, prop: &Property) -> TiledProperty {
    let property_type = match prop.property_type {
        PropertyType::String => "string",
        PropertyType::Int => "int",
        PropertyType::Float => "float",
        PropertyType::Bool => "bool",
        PropertyType::File => "file",
        PropertyType::Color => "color",
    };

    let value = if prop.property_type == PropertyType::Color {
        prop.value
            .as_str()
            .map(|s| serde_json::Value::String(export_color(s)))
            .unwrap_or_else(|| prop.value.clone())
    } else {
        prop.value.clone()
    };

    TiledProperty {
        name: name.to_string(),
        property_type: property_type.to_string(),
        value,
    }
}

/// Convert a cartile Properties map to a Vec<TiledProperty>.
fn export_properties(props: &cartile_format::Properties) -> Vec<TiledProperty> {
    let mut result: Vec<TiledProperty> = props
        .iter()
        .map(|(name, prop)| export_property(name, prop))
        .collect();
    // Sort for deterministic output
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Export a cartile Tileset to TiledTileset.
fn export_tileset(ts: &Tileset) -> TiledTileset {
    TiledTileset {
        firstgid: ts.first_gid,
        name: ts.name.clone(),
        tilewidth: ts.tile_width,
        tileheight: ts.tile_height,
        tilecount: ts.tile_count,
        columns: ts.columns,
        image: ts.image.clone(),
        imagewidth: ts.image_width,
        imageheight: ts.image_height,
        margin: ts.margin,
        spacing: ts.spacing,
        tiles: vec![],
        properties: vec![],
    }
}

/// Export a cartile TileLayer to a TiledTileLayer.
fn export_tile_layer(tl: &TileLayer, grid_width: u32, grid_height: u32) -> TiledTileLayer {
    let data: Vec<u32> = tl.data.iter().map(|tid| tid.raw()).collect();
    TiledTileLayer {
        name: tl.name.clone(),
        visible: tl.visible,
        opacity: tl.opacity,
        data,
        width: grid_width,
        height: grid_height,
        properties: export_properties(&tl.properties),
    }
}

/// Reverse-detect Tiled object fields from a cartile MapObject.
fn export_object(obj: &MapObject) -> TiledObject {
    let mut point = false;
    let mut ellipse = false;
    let mut polygon = None;
    let mut polyline = None;

    match obj.shape {
        Shape::Point => point = true,
        Shape::Ellipse => ellipse = true,
        Shape::Polygon => {
            polygon = obj
                .points
                .as_ref()
                .map(|pts| pts.iter().map(|p| TiledPoint { x: p.x, y: p.y }).collect());
        }
        Shape::Polyline => {
            polyline = obj
                .points
                .as_ref()
                .map(|pts| pts.iter().map(|p| TiledPoint { x: p.x, y: p.y }).collect());
        }
        Shape::Rect => {}
    }

    TiledObject {
        id: obj.id,
        name: obj.name.clone(),
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        rotation: obj.rotation,
        visible: true,
        point,
        ellipse,
        polygon,
        polyline,
        gid: None,
        text: None,
        template: None,
        properties: export_properties(&obj.properties),
    }
}

/// Export a cartile ObjectLayer to a TiledObjectLayer.
fn export_object_layer(ol: &ObjectLayer) -> TiledObjectLayer {
    TiledObjectLayer {
        name: ol.name.clone(),
        visible: ol.visible,
        opacity: ol.opacity,
        objects: ol.objects.iter().map(export_object).collect(),
        properties: export_properties(&ol.properties),
    }
}

/// Convert a `CartileMap` back to a Tiled JSON `TiledMap`.
///
/// Returns the TiledMap and a list of non-fatal warnings.
pub fn export_to_tiled(map: &CartileMap) -> anyhow::Result<(TiledMap, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();

    // Determine orientation from grid
    let (orientation, staggeraxis, staggerindex) = match (
        map.grid.grid_type,
        map.grid.projection.projection_type,
        &map.grid.stagger,
        &map.grid.orientation,
    ) {
        (GridType::Square, ProjectionType::Orthogonal, None, _) => {
            ("orthogonal".to_string(), None, None)
        }
        (GridType::Square, ProjectionType::Isometric, None, _) => {
            ("isometric".to_string(), None, None)
        }
        (GridType::Square, ProjectionType::Isometric, Some(stagger), _) => {
            let axis = match stagger.axis {
                StaggerAxis::X => "x",
                StaggerAxis::Y => "y",
            };
            let index = match stagger.index {
                cartile_format::StaggerIndex::Odd => "odd",
                cartile_format::StaggerIndex::Even => "even",
            };
            (
                "staggered".to_string(),
                Some(axis.to_string()),
                Some(index.to_string()),
            )
        }
        (GridType::Hexagonal, _, Some(stagger), hex_orient) => {
            let axis = match hex_orient {
                Some(cartile_format::HexOrientation::PointyTop) => "y",
                Some(cartile_format::HexOrientation::FlatTop) => "x",
                None => match stagger.axis {
                    StaggerAxis::X => "x",
                    StaggerAxis::Y => "y",
                },
            };
            let index = match stagger.index {
                cartile_format::StaggerIndex::Odd => "odd",
                cartile_format::StaggerIndex::Even => "even",
            };
            (
                "hexagonal".to_string(),
                Some(axis.to_string()),
                Some(index.to_string()),
            )
        }
        _ => {
            warnings.push(
                "could not determine Tiled orientation — defaulting to orthogonal".to_string(),
            );
            ("orthogonal".to_string(), None, None)
        }
    };

    // Convert tilesets
    let tilesets: Vec<TiledTilesetEntry> = map
        .tilesets
        .iter()
        .filter_map(|entry| match entry {
            TilesetEntry::Inline(ts) => Some(TiledTilesetEntry::Embedded(export_tileset(ts))),
            TilesetEntry::ExternalRef(r) => {
                warnings.push(format!(
                    "external tileset ref '{}' cannot be exported — skipped",
                    r.ref_path
                ));
                None
            }
        })
        .collect();

    // Convert layers
    let mut layers = Vec::new();
    for layer in &map.layers {
        match layer {
            Layer::Tile(tl) => {
                layers.push(TiledLayer::TileLayer(export_tile_layer(
                    tl,
                    map.grid.width,
                    map.grid.height,
                )));
            }
            Layer::Object(ol) => {
                layers.push(TiledLayer::ObjectGroup(export_object_layer(ol)));
            }
            Layer::Heightmap(hl) => {
                warnings.push(format!(
                    "heightmap layer '{}' has no Tiled equivalent — skipped",
                    hl.name
                ));
            }
        }
    }

    let properties = export_properties(&map.properties);

    let tiled_map = TiledMap {
        orientation,
        width: map.grid.width,
        height: map.grid.height,
        tilewidth: map.grid.tile_width,
        tileheight: map.grid.tile_height,
        infinite: false,
        tiledversion: Some("1.11.2".to_string()),
        renderorder: Some("right-down".to_string()),
        staggeraxis,
        staggerindex,
        hexsidelength: None,
        layers,
        tilesets,
        properties,
    };

    Ok((tiled_map, warnings))
}
