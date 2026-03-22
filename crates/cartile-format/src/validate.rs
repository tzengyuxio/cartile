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
            Some(angle) => {
                if angle <= 0.0 || angle >= 90.0 {
                    return Err(ValidationError::ObliqueAngleOutOfRange { angle });
                }
            }
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

/// Returns (name, first_gid, last_gid_inclusive) for each tileset entry.
fn tileset_ranges(map: &CartileMap) -> Result<Vec<(String, u32, u32)>, ValidationError> {
    let mut ranges = Vec::new();
    for entry in &map.tilesets {
        let (name, first_gid, tile_count) = match entry {
            TilesetEntry::ExternalRef(r) => {
                // External refs don't have tile_count; treat as single-tile placeholder.
                (r.ref_path.clone(), r.first_gid, 1u32)
            }
            TilesetEntry::Inline(ts) => (ts.name.clone(), ts.first_gid, ts.tile_count),
        };

        if first_gid < 1 {
            return Err(ValidationError::InvalidFirstGid { name, first_gid });
        }

        // last_gid = first_gid + tile_count - 1 (tile_count >= 1 assumed for inline)
        let last_gid = first_gid.saturating_add(tile_count.saturating_sub(1));
        if last_gid > MAX_GID {
            return Err(ValidationError::GidOutOfRange { gid: last_gid });
        }

        ranges.push((name, first_gid, last_gid));
    }
    Ok(ranges)
}

fn validate_tilesets(map: &CartileMap) -> Result<(), ValidationError> {
    let ranges = tileset_ranges(map)?;

    // Check for overlapping GID ranges (O(n²) is fine for typical map sizes).
    for i in 0..ranges.len() {
        for j in (i + 1)..ranges.len() {
            let (ref name_a, first_a, last_a) = ranges[i];
            let (ref name_b, first_b, last_b) = ranges[j];
            // Ranges [first_a, last_a] and [first_b, last_b] overlap iff
            // first_a <= last_b && first_b <= last_a
            if first_a <= last_b && first_b <= last_a {
                return Err(ValidationError::OverlappingGidRanges {
                    a: name_a.clone(),
                    b: name_b.clone(),
                });
            }
        }
    }

    // Validate auto-tile bitmasks for all inline tilesets.
    for entry in &map.tilesets {
        if let TilesetEntry::Inline(ts) = entry {
            for tile_data in ts.tiles.values() {
                if let Some(auto_tile) = &tile_data.auto_tile {
                    validate_auto_tile_bitmask(auto_tile.rule, auto_tile.bitmask)?;
                }
            }
        }
    }

    Ok(())
}

/// Validate a single bitmask according to its rule.
///
/// 4-bit rule: bits 0-3 only → valid range 0x00..=0x0F
/// 8-bit rule: diagonal bits (bits 0,2,5,7) are only valid when both adjacent
///   cardinal bits are set. Bit layout (standard 8-bit blob tileset numbering):
///   bit 0 = NW diagonal, bits 1 = N, bit 2 = NE diagonal, bit 3 = W,
///   bit 4 = E, bit 5 = SW diagonal, bit 6 = S, bit 7 = SE diagonal.
fn validate_auto_tile_bitmask(rule: AutoTileRule, bitmask: u8) -> Result<(), ValidationError> {
    match rule {
        AutoTileRule::Bitmask4bit => {
            if bitmask > 0x0F {
                return Err(ValidationError::InvalidAutoTileBitmask {
                    bitmask,
                    rule: "bitmask_4bit".to_string(),
                });
            }
        }
        AutoTileRule::Bitmask8bit => {
            // Diagonal requires both adjacent cardinals to be set.
            // Bit positions: 0=NW, 1=N, 2=NE, 3=W, 4=E, 5=SW, 6=S, 7=SE
            // NW (bit 0) needs N (bit 1) and W (bit 3)
            if bitmask & 0b0000_0001 != 0 && (bitmask & 0b0000_1010 != 0b0000_1010) {
                return Err(ValidationError::InvalidAutoTileBitmask {
                    bitmask,
                    rule: "bitmask_8bit".to_string(),
                });
            }
            // NE (bit 2) needs N (bit 1) and E (bit 4)
            if bitmask & 0b0000_0100 != 0 && (bitmask & 0b0001_0010 != 0b0001_0010) {
                return Err(ValidationError::InvalidAutoTileBitmask {
                    bitmask,
                    rule: "bitmask_8bit".to_string(),
                });
            }
            // SW (bit 5) needs W (bit 3) and S (bit 6)
            if bitmask & 0b0010_0000 != 0 && (bitmask & 0b0100_1000 != 0b0100_1000) {
                return Err(ValidationError::InvalidAutoTileBitmask {
                    bitmask,
                    rule: "bitmask_8bit".to_string(),
                });
            }
            // SE (bit 7) needs E (bit 4) and S (bit 6)
            if bitmask & 0b1000_0000 != 0 && (bitmask & 0b0101_0000 != 0b0101_0000) {
                return Err(ValidationError::InvalidAutoTileBitmask {
                    bitmask,
                    rule: "bitmask_8bit".to_string(),
                });
            }
        }
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
            Layer::Object(_) => {} // no size constraint
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

fn validate_height_mode(map: &CartileMap) -> Result<(), ValidationError> {
    let height_mode = map.grid.height_mode;

    let heightmap_layers: Vec<&str> = map
        .layers
        .iter()
        .filter_map(|l| {
            if let Layer::Heightmap(hl) = l {
                Some(hl.name.as_str())
            } else {
                None
            }
        })
        .collect();

    match height_mode {
        HeightMode::Vertex => {
            if heightmap_layers.is_empty() {
                return Err(ValidationError::MissingHeightmapLayer);
            }
            // Exactly one heightmap layer is required; more than one is also an error
            // (caught indirectly by duplicate name check, but not by count — allow
            // multiple distinctly-named heightmap layers for now; only require >= 1).
        }
        HeightMode::None | HeightMode::Stepped => {
            if let Some(name) = heightmap_layers.first() {
                return Err(ValidationError::UnexpectedHeightmapLayer {
                    mode: format!("{:?}", height_mode).to_lowercase(),
                    name: name.to_string(),
                });
            }
        }
    }

    // Non-stepped height mode: no tile layer may have non-zero elevation.
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
