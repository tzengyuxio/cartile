# Auto-tiling Runtime Design

> Date: 2026-03-22

## Overview

Add bitmask-based auto-tiling resolution to `cartile-format`. Given a tile layer with auto-tile groups defined in tilesets, automatically select the correct tile variant based on neighbor connectivity.

### Scope

- `bitmask_4bit` (4 cardinal neighbors, 16 combinations)
- `bitmask_8bit` (8 neighbors including diagonals, 47 valid combinations)
- Topology-aware neighbor lookup (bounded / wrap_x / wrap_y / wrap_xy)
- Fallback to bitmask 0 (isolated tile) when no match found

---

## 1. Module Location

New module `crates/cartile-format/src/autotile.rs`, exposed as `pub mod autotile` in `lib.rs`.

---

## 2. Public API

### `AutoTileIndex`

A precomputed lookup structure built from tileset auto-tile definitions.

```rust
pub struct AutoTileIndex {
    // group_name → { bitmask → local_tile_id }
    groups: HashMap<String, AutoTileGroup>,
}

struct AutoTileGroup {
    rule: AutoTileRule,
    variants: HashMap<u8, u32>,   // bitmask → local tile index
    fallback: Option<u32>,        // bitmask 0 tile (isolated)
}
```

### `build_autotile_index`

```rust
pub fn build_autotile_index(tilesets: &[TilesetEntry]) -> AutoTileIndex
```

Scans all inline tilesets' `tiles` for `auto_tile` entries. Groups them by `group` name. For each group, builds the bitmask → tile mapping. External ref tilesets are skipped (they don't have tile data inline).

### `resolve_autotiles`

```rust
pub fn resolve_autotiles(
    layer: &mut TileLayer,
    grid: &Grid,
    index: &AutoTileIndex,
    tilesets: &[TilesetEntry],
)
```

For each non-empty tile in the layer:
1. Determine which tileset it belongs to (by GID range)
2. Look up local tile index → check if it has an auto-tile group
3. Examine neighbors to compute bitmask
4. Look up bitmask in the index → replace tile with the correct variant
5. Preserve existing transform flags (flip/rotation)

---

## 3. Bitmask Computation

### 4-bit (cardinal neighbors)

```
      bit 0 (1)
        ↑
bit 3 (8) ← ■ → bit 1 (2)
        ↓
      bit 2 (4)
```

For each cardinal direction, if the neighbor tile belongs to the same auto-tile group, set the corresponding bit.

### 8-bit (including diagonals)

```
bit 0 (1)   bit 1 (2)   bit 2 (4)
    ↖           ↑           ↗
bit 3 (8)   ← ■ →   bit 4 (16)
    ↙           ↓           ↘
bit 5 (32)  bit 6 (64)  bit 7 (128)
```

Diagonal rule: a diagonal neighbor only counts if both adjacent cardinal neighbors are also connected. E.g., NW (bit 0) only counts if N (bit 1) AND W (bit 3) are both connected.

### Neighbor lookup with topology

| Topology | Out-of-bounds behavior |
|----------|----------------------|
| `bounded` | Treat as not connected (bit = 0) |
| `wrap_x` | x wraps: column -1 → last column, column = width → column 0 |
| `wrap_y` | y wraps: row -1 → last row, row = height → row 0 |
| `wrap_xy` | Both axes wrap |

---

## 4. Tile-to-Tileset Resolution

To determine which group a tile belongs to:

1. Extract GID from the tile (strip transform flags): `gid = raw & 0x1FFFFFFF`
2. Find the tileset whose range `[first_gid, first_gid + tile_count)` contains the GID
3. Compute local index: `local = gid - first_gid`
4. Look up `tileset.tiles[local.to_string()]` for auto-tile info

When replacing a tile with the correct variant:
1. Get the new local tile index from the bitmask lookup
2. Compute new GID: `new_gid = first_gid + new_local`
3. Preserve original transform flags: `new_raw = (original_raw & 0xE0000000) | new_gid`

---

## 5. Edge Cases

- **Tile not in any auto-tile group**: skip, leave unchanged
- **No matching bitmask in index**: use fallback (bitmask 0). If no fallback exists, leave unchanged
- **External ref tilesets**: skipped when building index (no tile data available). Tiles belonging to external tilesets are left unchanged.
- **Mixed groups in same tileset**: supported — different tiles in the same tileset can belong to different groups
- **Empty tiles (GID 0)**: never part of any group, never replaced

---

## 6. Testing Strategy

### Unit tests
- `build_autotile_index`: correct group extraction from tileset
- Bitmask computation for 4-bit: isolated tile (0), surrounded (15), L-shape, etc.
- Bitmask computation for 8-bit: diagonal rule enforcement
- Neighbor lookup with all topology modes
- Tile replacement preserves transform flags

### Integration tests
- Full `resolve_autotiles` on a small map with known expected output
- Map with wrap topology
- Map with tiles from multiple tilesets and groups
