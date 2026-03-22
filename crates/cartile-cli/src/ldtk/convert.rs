use std::collections::HashMap;

use anyhow::Context;
use cartile_format::{
    CartileMap, Grid, GridType, HeightMode, Layer, MapObject, ObjectLayer, Projection,
    ProjectionType, Shape, TileId, TileLayer, Tileset, TilesetEntry, Topology,
};

use super::types::{LdtkGridTile, LdtkLayerInstance, LdtkLevel, LdtkRoot, LdtkTilesetDef};

const FLIP_HORIZONTAL: u32 = 0x8000_0000;
const FLIP_VERTICAL: u32 = 0x4000_0000;

/// Check if a JSON string looks like an LDtk project by inspecting the `__header__` field.
pub fn is_ldtk_json(json: &str) -> bool {
    // Quick heuristic: check for the LDtk header marker
    json.contains("\"__header__\"") && json.contains("\"LDtk\"")
}

/// Convert an LDtk project to one or more CartileMaps.
///
/// If the project has multiple levels, only the first level is converted
/// (or a specific level can be selected by name). Returns the map and warnings.
pub fn convert_ldtk_project(
    root: &LdtkRoot,
    level_name: Option<&str>,
) -> anyhow::Result<(CartileMap, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();

    // Collect all levels (from root.levels or from worlds)
    let all_levels: Vec<&LdtkLevel> = if !root.levels.is_empty() {
        root.levels.iter().collect()
    } else {
        root.worlds.iter().flat_map(|w| w.levels.iter()).collect()
    };

    if all_levels.is_empty() {
        anyhow::bail!("LDtk project has no levels");
    }

    // Select the target level
    let level = if let Some(name) = level_name {
        all_levels
            .iter()
            .find(|l| l.identifier == name)
            .with_context(|| format!("level '{name}' not found in LDtk project"))?
    } else {
        if all_levels.len() > 1 {
            warnings.push(format!(
                "LDtk project has {} levels — converting only the first ('{}'); \
                 use --level to select a different one",
                all_levels.len(),
                all_levels[0].identifier
            ));
        }
        all_levels[0]
    };

    let layer_instances = level
        .layer_instances
        .as_ref()
        .context("level has no layerInstances (separate level files not supported)")?;

    // Determine grid size from the first layer, or fall back to project default
    let grid_size = layer_instances
        .first()
        .map(|l| l.grid_size)
        .unwrap_or(root.default_grid_size);

    let grid_w = level.px_wid / grid_size;
    let grid_h = level.px_hei / grid_size;

    // Build tileset map: uid → definition
    let tileset_map: HashMap<u32, &LdtkTilesetDef> =
        root.defs.tilesets.iter().map(|ts| (ts.uid, ts)).collect();

    // Convert tilesets and build a mapping from LDtk tileset UID → firstgid
    let mut tilesets = Vec::new();
    let mut uid_to_firstgid: HashMap<u32, u32> = HashMap::new();
    let mut next_gid: u32 = 1;

    // Only include tilesets that are actually referenced by layers
    let referenced_uids: Vec<u32> = layer_instances
        .iter()
        .filter_map(|l| l.tileset_def_uid)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for &uid in &referenced_uids {
        if let Some(ts_def) = tileset_map.get(&uid) {
            let cartile_ts = convert_tileset(ts_def, next_gid);
            uid_to_firstgid.insert(uid, next_gid);
            next_gid += cartile_ts.tile_count;
            tilesets.push(TilesetEntry::Inline(cartile_ts));
        }
    }

    // Convert layers (LDtk layers are in render order: last = bottom, first = top)
    // We reverse to match cartile's bottom-to-top convention
    let mut layers = Vec::new();
    for li in layer_instances.iter().rev() {
        match li.layer_type.as_str() {
            "Tiles" => {
                if let Some(tile_layer) = convert_tile_layer(li, &uid_to_firstgid, &mut warnings) {
                    layers.push(Layer::Tile(tile_layer));
                }
            }
            "IntGrid" => {
                // IntGrid layers may also have auto-layer tiles
                if !li.auto_layer_tiles.is_empty() {
                    if let Some(tile_layer) =
                        convert_auto_layer(li, &uid_to_firstgid, &mut warnings)
                    {
                        layers.push(Layer::Tile(tile_layer));
                    }
                } else if !li.int_grid_csv.is_empty() {
                    layers.push(Layer::Tile(convert_intgrid_layer(li)));
                }
            }
            "AutoLayer" => {
                if let Some(tile_layer) = convert_auto_layer(li, &uid_to_firstgid, &mut warnings) {
                    layers.push(Layer::Tile(tile_layer));
                }
            }
            "Entities" => {
                let obj_layer = convert_entity_layer(li);
                layers.push(Layer::Object(obj_layer));
            }
            other => {
                warnings.push(format!(
                    "unknown LDtk layer type '{other}' for '{}' — skipped",
                    li.identifier
                ));
            }
        }
    }

    let map = CartileMap {
        cartile: "0.1.0".to_string(),
        map_type: "map".to_string(),
        name: level.identifier.clone(),
        properties: HashMap::new(),
        grid: Grid {
            grid_type: GridType::Square,
            width: grid_w,
            height: grid_h,
            tile_width: grid_size,
            tile_height: grid_size,
            orientation: None,
            stagger: None,
            topology: Topology::Bounded,
            projection: Projection {
                projection_type: ProjectionType::Orthogonal,
                angle: None,
                extra: HashMap::new(),
            },
            height_mode: HeightMode::None,
            extra: HashMap::new(),
        },
        tilesets,
        layers,
        extra: HashMap::new(),
    };

    Ok((map, warnings))
}

/// Convert an LDtk tileset definition to a cartile Tileset.
fn convert_tileset(ts_def: &LdtkTilesetDef, first_gid: u32) -> Tileset {
    let columns = if ts_def.tile_grid_size > 0 {
        (ts_def.px_wid - ts_def.padding * 2 + ts_def.spacing)
            / (ts_def.tile_grid_size + ts_def.spacing)
    } else {
        1
    };
    let rows = if ts_def.tile_grid_size > 0 {
        (ts_def.px_hei - ts_def.padding * 2 + ts_def.spacing)
            / (ts_def.tile_grid_size + ts_def.spacing)
    } else {
        1
    };

    Tileset {
        name: ts_def.identifier.clone(),
        tile_width: ts_def.tile_grid_size,
        tile_height: ts_def.tile_grid_size,
        image: ts_def.rel_path.clone().unwrap_or_default(),
        image_width: ts_def.px_wid,
        image_height: ts_def.px_hei,
        columns,
        tile_count: columns * rows,
        margin: ts_def.padding,
        spacing: ts_def.spacing,
        first_gid,
        tiles: HashMap::new(),
        extra: HashMap::new(),
    }
}

/// Convert LDtk flip bits (0-3) to cartile flag bits.
fn ldtk_flip_to_flags(flip: u32) -> u32 {
    let mut flags = 0u32;
    if flip & 1 != 0 {
        flags |= FLIP_HORIZONTAL;
    }
    if flip & 2 != 0 {
        flags |= FLIP_VERTICAL;
    }
    flags
}

/// Convert a Tiles layer (gridTiles) to a cartile TileLayer.
fn convert_tile_layer(
    li: &LdtkLayerInstance,
    uid_to_firstgid: &HashMap<u32, u32>,
    warnings: &mut Vec<String>,
) -> Option<TileLayer> {
    let first_gid = match li.tileset_def_uid {
        Some(uid) => *uid_to_firstgid.get(&uid)?,
        None => {
            warnings.push(format!(
                "Tiles layer '{}' has no tileset — skipped",
                li.identifier
            ));
            return None;
        }
    };

    let data = grid_tiles_to_dense(&li.grid_tiles, li.c_wid, li.c_hei, li.grid_size, first_gid);

    Some(TileLayer {
        name: li.identifier.clone(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data,
        properties: HashMap::new(),
    })
}

/// Convert an AutoLayer (autoLayerTiles) to a cartile TileLayer.
fn convert_auto_layer(
    li: &LdtkLayerInstance,
    uid_to_firstgid: &HashMap<u32, u32>,
    warnings: &mut Vec<String>,
) -> Option<TileLayer> {
    let first_gid = match li.tileset_def_uid {
        Some(uid) => *uid_to_firstgid.get(&uid)?,
        None => {
            warnings.push(format!(
                "AutoLayer '{}' has no tileset — skipped",
                li.identifier
            ));
            return None;
        }
    };

    let data = grid_tiles_to_dense(
        &li.auto_layer_tiles,
        li.c_wid,
        li.c_hei,
        li.grid_size,
        first_gid,
    );

    Some(TileLayer {
        name: li.identifier.clone(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data,
        properties: HashMap::new(),
    })
}

/// Convert LDtk gridTiles (sparse px-based) to a dense row-major tile array.
fn grid_tiles_to_dense(
    tiles: &[LdtkGridTile],
    c_wid: u32,
    c_hei: u32,
    grid_size: u32,
    first_gid: u32,
) -> Vec<TileId> {
    let total = (c_wid * c_hei) as usize;
    let mut data = vec![TileId::EMPTY; total];

    for tile in tiles {
        let col = tile.px[0] as u32 / grid_size;
        let row = tile.px[1] as u32 / grid_size;
        if col < c_wid && row < c_hei {
            let idx = (row * c_wid + col) as usize;
            // LDtk tile_id is 0-based within the tileset; add first_gid to get global ID
            let gid = tile.tile_id + first_gid;
            let flags = ldtk_flip_to_flags(tile.flip);
            data[idx] = TileId::from_raw(flags | gid);
        }
    }

    data
}

/// Convert an IntGrid layer (intGridCsv) to a cartile TileLayer.
/// IntGrid values are stored directly as GIDs (no tileset offset).
fn convert_intgrid_layer(li: &LdtkLayerInstance) -> TileLayer {
    let data: Vec<TileId> = li
        .int_grid_csv
        .iter()
        .map(|&v| {
            if v <= 0 {
                TileId::EMPTY
            } else {
                TileId::from_gid(v as u32)
            }
        })
        .collect();

    TileLayer {
        name: li.identifier.clone(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data,
        properties: HashMap::new(),
    }
}

/// Convert an Entities layer to a cartile ObjectLayer.
fn convert_entity_layer(li: &LdtkLayerInstance) -> ObjectLayer {
    let mut objects = Vec::new();
    for (i, entity) in li.entity_instances.iter().enumerate() {
        objects.push(MapObject {
            id: (i + 1) as u64,
            name: entity.identifier.clone(),
            x: entity.px[0],
            y: entity.px[1],
            width: entity.width,
            height: entity.height,
            shape: if entity.width > 0.0 || entity.height > 0.0 {
                Shape::Rect
            } else {
                Shape::Point
            },
            rotation: 0.0,
            points: None,
            properties: HashMap::new(),
            extra: HashMap::new(),
        });
    }

    ObjectLayer {
        name: li.identifier.clone(),
        visible: true,
        opacity: 1.0,
        objects,
        properties: HashMap::new(),
    }
}
