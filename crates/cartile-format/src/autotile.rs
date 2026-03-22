use std::collections::HashMap;

use crate::tile_id::TileId;
use crate::types::grid::{Grid, Topology};
use crate::types::layer::TileLayer;
use crate::types::tileset::{AutoTileRule, TilesetEntry};

/// Per-group auto-tile data: rule type, bitmask→local_tile_id mapping, and fallback.
#[derive(Debug, Clone)]
struct AutoTileGroup {
    rule: AutoTileRule,
    /// bitmask value → local tile index within the tileset
    variants: HashMap<u8, u32>,
    /// The local tile index for bitmask 0 (isolated), used as fallback.
    fallback: Option<u32>,
}

/// Precomputed lookup structure for auto-tile resolution.
///
/// Built from tileset definitions, maps group names to their bitmask→tile mappings.
#[derive(Debug, Clone)]
pub struct AutoTileIndex {
    groups: HashMap<String, AutoTileGroup>,
}

impl AutoTileIndex {
    /// Returns true if the given group name exists in the index.
    pub fn has_group(&self, group: &str) -> bool {
        self.groups.contains_key(group)
    }

    /// Returns the rule type for a given group, if it exists.
    pub fn group_rule(&self, group: &str) -> Option<AutoTileRule> {
        self.groups.get(group).map(|g| g.rule)
    }

    /// Returns the number of variant mappings for a given group.
    pub fn group_variant_count(&self, group: &str) -> usize {
        self.groups.get(group).map_or(0, |g| g.variants.len())
    }

    /// Look up the local tile index for a group + bitmask, falling back to bitmask 0.
    fn lookup(&self, group: &str, bitmask: u8) -> Option<u32> {
        let g = self.groups.get(group)?;
        if let Some(&local) = g.variants.get(&bitmask) {
            Some(local)
        } else {
            g.fallback
        }
    }
}

/// Scan all inline tilesets for auto-tile entries and build a lookup index.
///
/// External ref tilesets are skipped since they don't contain tile data inline.
pub fn build_autotile_index(tilesets: &[TilesetEntry]) -> AutoTileIndex {
    let mut groups: HashMap<String, AutoTileGroup> = HashMap::new();

    for entry in tilesets {
        let tileset = match entry {
            TilesetEntry::Inline(ts) => ts,
            TilesetEntry::ExternalRef(_) => continue,
        };

        for (key, tile_data) in &tileset.tiles {
            let auto_tile = match &tile_data.auto_tile {
                Some(at) => at,
                None => continue,
            };

            let local_index: u32 = match key.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let group = groups
                .entry(auto_tile.group.clone())
                .or_insert_with(|| AutoTileGroup {
                    rule: auto_tile.rule,
                    variants: HashMap::new(),
                    fallback: None,
                });

            group.variants.insert(auto_tile.bitmask, local_index);

            if auto_tile.bitmask == 0 {
                group.fallback = Some(local_index);
            }
        }
    }

    AutoTileIndex { groups }
}

/// Resolve neighbor position with topology-aware wrapping.
///
/// Returns `None` if the position is out of bounds (for bounded topology),
/// or the wrapped coordinates for wrapping topologies.
fn get_neighbor_pos(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    topology: Topology,
) -> Option<(u32, u32)> {
    let w = width as i32;
    let h = height as i32;

    let wrap_x = matches!(topology, Topology::WrapX | Topology::WrapXy);
    let wrap_y = matches!(topology, Topology::WrapY | Topology::WrapXy);

    let nx = if x < 0 || x >= w {
        if wrap_x {
            x.rem_euclid(w)
        } else {
            return None;
        }
    } else {
        x
    };

    let ny = if y < 0 || y >= h {
        if wrap_y {
            y.rem_euclid(h)
        } else {
            return None;
        }
    } else {
        y
    };

    Some((nx as u32, ny as u32))
}

/// Info about a tile's tileset membership and auto-tile group.
struct TileInfo {
    first_gid: u32,
    group: String,
}

/// Find which tileset a GID belongs to and return its auto-tile group info.
fn find_tile_info(gid: u32, tilesets: &[TilesetEntry]) -> Option<TileInfo> {
    for entry in tilesets {
        let tileset = match entry {
            TilesetEntry::Inline(ts) => ts,
            TilesetEntry::ExternalRef(_) => continue,
        };

        if gid >= tileset.first_gid && gid < tileset.first_gid + tileset.tile_count {
            let local = gid - tileset.first_gid;
            if let Some(tile_data) = tileset.tiles.get(&local.to_string())
                && let Some(auto_tile) = &tile_data.auto_tile
            {
                return Some(TileInfo {
                    first_gid: tileset.first_gid,
                    group: auto_tile.group.clone(),
                });
            }
            return None;
        }
    }
    None
}

/// Check if the tile at a given position belongs to the specified auto-tile group.
fn neighbor_connects(
    layer: &TileLayer,
    grid_width: u32,
    nx: u32,
    ny: u32,
    group: &str,
    tilesets: &[TilesetEntry],
) -> bool {
    let idx = (ny * grid_width + nx) as usize;
    let tile = layer.data[idx];
    let gid = tile.gid();
    if gid == 0 {
        return false;
    }
    match find_tile_info(gid, tilesets) {
        Some(info) => info.group == group,
        None => false,
    }
}

/// Compute the 4-bit bitmask for a tile at (cx, cy).
///
/// Bit assignment: N=1, E=2, S=4, W=8
fn compute_bitmask_4bit(
    layer: &TileLayer,
    grid: &Grid,
    cx: u32,
    cy: u32,
    group: &str,
    tilesets: &[TilesetEntry],
) -> u8 {
    let x = cx as i32;
    let y = cy as i32;
    let w = grid.width;
    let h = grid.height;
    let topo = grid.topology;

    let mut mask: u8 = 0;

    // N (y-1)
    if let Some((nx, ny)) = get_neighbor_pos(x, y - 1, w, h, topo)
        && neighbor_connects(layer, w, nx, ny, group, tilesets)
    {
        mask |= 1;
    }
    // E (x+1)
    if let Some((nx, ny)) = get_neighbor_pos(x + 1, y, w, h, topo)
        && neighbor_connects(layer, w, nx, ny, group, tilesets)
    {
        mask |= 2;
    }
    // S (y+1)
    if let Some((nx, ny)) = get_neighbor_pos(x, y + 1, w, h, topo)
        && neighbor_connects(layer, w, nx, ny, group, tilesets)
    {
        mask |= 4;
    }
    // W (x-1)
    if let Some((nx, ny)) = get_neighbor_pos(x - 1, y, w, h, topo)
        && neighbor_connects(layer, w, nx, ny, group, tilesets)
    {
        mask |= 8;
    }

    mask
}

/// Compute the 8-bit bitmask for a tile at (cx, cy).
///
/// Bit assignment: NW=1, N=2, NE=4, W=8, E=16, SW=32, S=64, SE=128
///
/// Diagonal rule: a diagonal only counts if both adjacent cardinals are connected.
fn compute_bitmask_8bit(
    layer: &TileLayer,
    grid: &Grid,
    cx: u32,
    cy: u32,
    group: &str,
    tilesets: &[TilesetEntry],
) -> u8 {
    let x = cx as i32;
    let y = cy as i32;
    let w = grid.width;
    let h = grid.height;
    let topo = grid.topology;

    // Check cardinal directions first
    let connects = |dx: i32, dy: i32| -> bool {
        get_neighbor_pos(x + dx, y + dy, w, h, topo)
            .is_some_and(|(nx, ny)| neighbor_connects(layer, w, nx, ny, group, tilesets))
    };

    let n = connects(0, -1);
    let e = connects(1, 0);
    let s = connects(0, 1);
    let w_dir = connects(-1, 0);

    let mut mask: u8 = 0;

    // Cardinals
    if n {
        mask |= 2;
    }
    if e {
        mask |= 16;
    }
    if s {
        mask |= 64;
    }
    if w_dir {
        mask |= 8;
    }

    // Diagonals (only if both adjacent cardinals connect)
    if n && w_dir && connects(-1, -1) {
        mask |= 1; // NW
    }
    if n && e && connects(1, -1) {
        mask |= 4; // NE
    }
    if s && w_dir && connects(-1, 1) {
        mask |= 32; // SW
    }
    if s && e && connects(1, 1) {
        mask |= 128; // SE
    }

    mask
}

/// Resolve auto-tiles in a tile layer, replacing each auto-tile with the
/// correct variant based on neighbor connectivity.
///
/// For each tile that belongs to an auto-tile group:
/// 1. Compute the bitmask based on neighbors
/// 2. Look up the correct variant in the index
/// 3. Replace the tile, preserving transform flags
pub fn resolve_autotiles(
    layer: &mut TileLayer,
    grid: &Grid,
    index: &AutoTileIndex,
    tilesets: &[TilesetEntry],
) {
    let w = grid.width;
    let h = grid.height;

    // Take ownership of data so we can read from a snapshot while mutating
    let snapshot = layer.data.clone();

    // Build a snapshot layer for neighbor lookups (reads from original state)
    let snap_layer = TileLayer {
        name: layer.name.clone(),
        visible: layer.visible,
        opacity: layer.opacity,
        elevation: layer.elevation,
        encoding: layer.encoding.clone(),
        data: snapshot,
        properties: layer.properties.clone(),
    };

    for cy in 0..h {
        for cx in 0..w {
            let idx = (cy * w + cx) as usize;
            let tile = snap_layer.data[idx];
            let gid = tile.gid();
            if gid == 0 {
                continue;
            }

            let info = match find_tile_info(gid, tilesets) {
                Some(info) => info,
                None => continue,
            };

            let group_data = match index.groups.get(&info.group) {
                Some(g) => g,
                None => continue,
            };

            let bitmask = match group_data.rule {
                AutoTileRule::Bitmask4bit => {
                    compute_bitmask_4bit(&snap_layer, grid, cx, cy, &info.group, tilesets)
                }
                AutoTileRule::Bitmask8bit => {
                    compute_bitmask_8bit(&snap_layer, grid, cx, cy, &info.group, tilesets)
                }
            };

            if let Some(new_local) = index.lookup(&info.group, bitmask) {
                let new_gid = info.first_gid + new_local;
                let new_raw = tile.flags() | new_gid;
                layer.data[idx] = TileId::from_raw(new_raw);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_neighbor_pos_bounded_out_of_bounds() {
        assert_eq!(get_neighbor_pos(-1, 0, 5, 5, Topology::Bounded), None);
        assert_eq!(get_neighbor_pos(0, -1, 5, 5, Topology::Bounded), None);
        assert_eq!(get_neighbor_pos(5, 0, 5, 5, Topology::Bounded), None);
        assert_eq!(get_neighbor_pos(0, 5, 5, 5, Topology::Bounded), None);
    }

    #[test]
    fn get_neighbor_pos_bounded_in_bounds() {
        assert_eq!(
            get_neighbor_pos(0, 0, 5, 5, Topology::Bounded),
            Some((0, 0))
        );
        assert_eq!(
            get_neighbor_pos(4, 4, 5, 5, Topology::Bounded),
            Some((4, 4))
        );
    }

    #[test]
    fn get_neighbor_pos_wrap_x() {
        assert_eq!(get_neighbor_pos(-1, 0, 5, 5, Topology::WrapX), Some((4, 0)));
        assert_eq!(get_neighbor_pos(5, 0, 5, 5, Topology::WrapX), Some((0, 0)));
        // y does not wrap
        assert_eq!(get_neighbor_pos(0, -1, 5, 5, Topology::WrapX), None);
    }

    #[test]
    fn get_neighbor_pos_wrap_y() {
        assert_eq!(get_neighbor_pos(0, -1, 5, 5, Topology::WrapY), Some((0, 4)));
        assert_eq!(get_neighbor_pos(0, 5, 5, 5, Topology::WrapY), Some((0, 0)));
        // x does not wrap
        assert_eq!(get_neighbor_pos(-1, 0, 5, 5, Topology::WrapY), None);
    }

    #[test]
    fn get_neighbor_pos_wrap_xy() {
        assert_eq!(
            get_neighbor_pos(-1, -1, 5, 5, Topology::WrapXy),
            Some((4, 4))
        );
        assert_eq!(get_neighbor_pos(5, 5, 5, 5, Topology::WrapXy), Some((0, 0)));
    }
}
