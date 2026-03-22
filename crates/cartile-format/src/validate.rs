use std::collections::HashSet;

use crate::error::ValidationError;
use crate::types::grid::{GridType, HeightMode, ProjectionType};
use crate::types::layer::Layer;
use crate::types::map::CartileMap;
use crate::types::tileset::{AutoTileRule, TilesetEntry};

impl CartileMap {
    /// Perform semantic validation of the map.
    ///
    /// Returns `Ok(())` if all checks pass, or the first `ValidationError` found.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_grid(self)?;
        validate_tilesets(self)?;
        validate_layers(self)?;
        validate_height_mode(self)?;
        validate_object_ids(self)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Grid validation
// ---------------------------------------------------------------------------

fn validate_grid(map: &CartileMap) -> Result<(), ValidationError> {
    let grid = &map.grid;

    match grid.grid_type {
        GridType::Hexagonal => {
            if grid.orientation.is_none() {
                return Err(ValidationError::MissingHexOrientation);
            }
            if grid.stagger.is_none() {
                return Err(ValidationError::MissingHexStagger);
            }
        }
        GridType::Square => {
            if grid.orientation.is_some() {
                return Err(ValidationError::UnexpectedOrientation);
            }
        }
    }

    match grid.projection.projection_type {
        ProjectionType::Oblique => match grid.projection.angle {
            None => return Err(ValidationError::MissingObliqueAngle),
            Some(angle) if angle <= 0.0 || angle >= 90.0 => {
                return Err(ValidationError::ObliqueAngleOutOfRange { angle });
            }
            _ => {}
        },
        _ => {
            if grid.projection.angle.is_some() {
                return Err(ValidationError::UnexpectedProjectionAngle);
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tileset validation
// ---------------------------------------------------------------------------

const MAX_GID: u32 = 0x1FFF_FFFF;

fn validate_tilesets(map: &CartileMap) -> Result<(), ValidationError> {
    // Collect GID ranges, borrowing names to avoid allocation on the happy path.
    let mut ranges: Vec<(&str, u32, u32)> = Vec::new();

    for entry in &map.tilesets {
        let (name, first_gid, tile_count) = match entry {
            TilesetEntry::ExternalRef(r) => {
                // External refs don't have tile_count; treat as single-tile placeholder.
                (r.ref_path.as_str(), r.first_gid, 1u32)
            }
            TilesetEntry::Inline(ts) => (ts.name.as_str(), ts.first_gid, ts.tile_count),
        };

        if first_gid < 1 {
            return Err(ValidationError::InvalidFirstGid {
                name: name.to_string(),
                first_gid,
            });
        }

        let last_gid = first_gid.saturating_add(tile_count.saturating_sub(1));
        if last_gid > MAX_GID {
            return Err(ValidationError::GidOutOfRange { gid: last_gid });
        }

        ranges.push((name, first_gid, last_gid));

        // Validate auto-tile bitmasks for inline tilesets.
        if let TilesetEntry::Inline(ts) = entry {
            for tile_data in ts.tiles.values() {
                if let Some(auto_tile) = &tile_data.auto_tile {
                    validate_auto_tile_bitmask(auto_tile.rule, auto_tile.bitmask)?;
                }
            }
        }
    }

    // Check for overlapping GID ranges (O(n²) is fine for typical map sizes).
    for i in 0..ranges.len() {
        for j in (i + 1)..ranges.len() {
            let (name_a, first_a, last_a) = ranges[i];
            let (name_b, first_b, last_b) = ranges[j];
            if first_a <= last_b && first_b <= last_a {
                return Err(ValidationError::OverlappingGidRanges {
                    a: name_a.to_string(),
                    b: name_b.to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_auto_tile_bitmask(rule: AutoTileRule, bitmask: u8) -> Result<(), ValidationError> {
    let valid = match rule {
        AutoTileRule::Bitmask4bit => bitmask <= 0x0F,
        AutoTileRule::Bitmask8bit => {
            // Diagonal requires both adjacent cardinals to be set.
            // Bit positions: 0=NW, 1=N, 2=NE, 3=W, 4=E, 5=SW, 6=S, 7=SE
            let check = |diagonal_bit: u8, cardinal_mask: u8| {
                bitmask & diagonal_bit == 0 || bitmask & cardinal_mask == cardinal_mask
            };
            check(0b0000_0001, 0b0000_1010)  // NW needs N+W
                && check(0b0000_0100, 0b0001_0010)  // NE needs N+E
                && check(0b0010_0000, 0b0100_1000)  // SW needs W+S
                && check(0b1000_0000, 0b0101_0000) // SE needs E+S
        }
    };
    if !valid {
        return Err(ValidationError::InvalidAutoTileBitmask { bitmask, rule });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Layer validation
// ---------------------------------------------------------------------------

fn validate_layers(map: &CartileMap) -> Result<(), ValidationError> {
    let expected_tile = (map.grid.width * map.grid.height) as usize;
    let expected_hmap = ((map.grid.width + 1) * (map.grid.height + 1)) as usize;

    let mut seen_names: HashSet<&str> = HashSet::new();

    for layer in &map.layers {
        let name = layer_name(layer);

        if !seen_names.insert(name) {
            return Err(ValidationError::DuplicateLayerName {
                name: name.to_string(),
            });
        }

        match layer {
            Layer::Tile(tl) => {
                if tl.data.len() != expected_tile {
                    return Err(ValidationError::InvalidDataLength {
                        name: tl.name.clone(),
                        actual: tl.data.len(),
                        expected: expected_tile,
                    });
                }
            }
            Layer::Heightmap(hl) => {
                if hl.data.len() != expected_hmap {
                    return Err(ValidationError::InvalidHeightmapLength {
                        name: hl.name.clone(),
                        actual: hl.data.len(),
                        expected: expected_hmap,
                    });
                }
            }
            Layer::Object(_) => {}
        }
    }

    Ok(())
}

fn layer_name(layer: &Layer) -> &str {
    match layer {
        Layer::Tile(tl) => &tl.name,
        Layer::Object(ol) => &ol.name,
        Layer::Heightmap(hl) => &hl.name,
    }
}

// ---------------------------------------------------------------------------
// Height mode validation
// ---------------------------------------------------------------------------

fn height_mode_str(mode: HeightMode) -> &'static str {
    match mode {
        HeightMode::None => "none",
        HeightMode::Stepped => "stepped",
        HeightMode::Vertex => "vertex",
    }
}

fn validate_height_mode(map: &CartileMap) -> Result<(), ValidationError> {
    let height_mode = map.grid.height_mode;

    // Find the first heightmap layer (no Vec allocation needed).
    let first_heightmap = map.layers.iter().find_map(|l| {
        if let Layer::Heightmap(hl) = l {
            Some(hl.name.as_str())
        } else {
            None
        }
    });

    match height_mode {
        HeightMode::Vertex => {
            if first_heightmap.is_none() {
                return Err(ValidationError::MissingHeightmapLayer);
            }
        }
        HeightMode::None | HeightMode::Stepped => {
            if let Some(name) = first_heightmap {
                return Err(ValidationError::UnexpectedHeightmapLayer {
                    mode: height_mode_str(height_mode).to_string(),
                    name: name.to_string(),
                });
            }
        }
    }

    if height_mode != HeightMode::Stepped {
        for layer in &map.layers {
            if let Layer::Tile(tl) = layer
                && tl.elevation != 0
            {
                return Err(ValidationError::UnexpectedElevation {
                    name: tl.name.clone(),
                });
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Object ID validation
// ---------------------------------------------------------------------------

fn validate_object_ids(map: &CartileMap) -> Result<(), ValidationError> {
    let mut seen_ids: HashSet<u64> = HashSet::new();

    for layer in &map.layers {
        if let Layer::Object(ol) = layer {
            for obj in &ol.objects {
                if !seen_ids.insert(obj.id) {
                    return Err(ValidationError::DuplicateObjectId { id: obj.id });
                }
            }
        }
    }

    Ok(())
}
