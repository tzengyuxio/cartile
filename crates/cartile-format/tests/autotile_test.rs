use std::collections::HashMap;

use cartile_format::autotile::{build_autotile_index, resolve_autotiles};
use cartile_format::tile_id::TileId;
use cartile_format::types::grid::*;
use cartile_format::types::layer::TileLayer;
use cartile_format::types::tileset::*;

/// Create a 4-bit auto-tile tileset with 16 tiles for the "grass" group.
/// Tile local index N has bitmask N (0..=15).
fn make_4bit_tileset() -> Tileset {
    let mut tiles = HashMap::new();
    for bitmask in 0u8..=15 {
        tiles.insert(
            bitmask.to_string(),
            TileData {
                properties: HashMap::new(),
                auto_tile: Some(AutoTile {
                    group: "grass".to_string(),
                    rule: AutoTileRule::Bitmask4bit,
                    bitmask,
                }),
                extra: HashMap::new(),
            },
        );
    }
    Tileset {
        name: "terrain".to_string(),
        tile_width: 16,
        tile_height: 16,
        image: "terrain.png".to_string(),
        image_width: 256,
        image_height: 256,
        columns: 16,
        tile_count: 16,
        margin: 0,
        spacing: 0,
        first_gid: 1,
        tiles,
        extra: HashMap::new(),
    }
}

/// Create an 8-bit auto-tile tileset with 256 tiles for the "stone" group.
/// Tile local index N has bitmask N (0..=255).
fn make_8bit_tileset() -> Tileset {
    let mut tiles = HashMap::new();
    for bitmask in 0u8..=255u8 {
        tiles.insert(
            (bitmask as u32).to_string(),
            TileData {
                properties: HashMap::new(),
                auto_tile: Some(AutoTile {
                    group: "stone".to_string(),
                    rule: AutoTileRule::Bitmask8bit,
                    bitmask,
                }),
                extra: HashMap::new(),
            },
        );
    }
    Tileset {
        name: "stone_set".to_string(),
        tile_width: 16,
        tile_height: 16,
        image: "stone.png".to_string(),
        image_width: 256,
        image_height: 256,
        columns: 16,
        tile_count: 256,
        margin: 0,
        spacing: 0,
        first_gid: 100,
        tiles,
        extra: HashMap::new(),
    }
}

fn make_bounded_grid(width: u32, height: u32) -> Grid {
    Grid {
        grid_type: GridType::Square,
        width,
        height,
        tile_width: 16,
        tile_height: 16,
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
    }
}

fn make_grid_with_topology(width: u32, height: u32, topology: Topology) -> Grid {
    let mut g = make_bounded_grid(width, height);
    g.topology = topology;
    g
}

fn make_tile_layer(name: &str, data: Vec<TileId>) -> TileLayer {
    TileLayer {
        name: name.to_string(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data,
        properties: HashMap::new(),
    }
}

// ============================================================
// Tests
// ============================================================

#[test]
fn build_index_from_tileset() {
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);

    assert!(index.has_group("grass"));
    assert!(!index.has_group("water"));
    assert_eq!(index.group_rule("grass"), Some(AutoTileRule::Bitmask4bit));
    assert_eq!(index.group_variant_count("grass"), 16);
}

#[test]
fn build_index_skips_external_ref() {
    let ext = TilesetEntry::ExternalRef(TilesetRef {
        ref_path: "terrain.cartile-ts".to_string(),
        first_gid: 1,
    });
    let index = build_autotile_index(&[ext]);
    assert!(!index.has_group("grass"));
}

#[test]
fn bitmask_4bit_isolated() {
    // 3x3 grid, only center tile is grass, rest empty
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 3);

    let e = TileId::EMPTY;
    let g = TileId::from_gid(1); // grass tile (local 0, gid = first_gid + 0 = 1)

    #[rustfmt::skip]
    let mut layer = make_tile_layer("ground", vec![
        e, e, e,
        e, g, e,
        e, e, e,
    ]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Center tile should get bitmask 0 (isolated) → local 0 → gid 1
    assert_eq!(layer.data[4].gid(), 1);
}

#[test]
fn bitmask_4bit_surrounded() {
    // 3x3 grid, all tiles are grass
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 3);

    let g = TileId::from_gid(1); // grass local 0

    #[rustfmt::skip]
    let mut layer = make_tile_layer("ground", vec![
        g, g, g,
        g, g, g,
        g, g, g,
    ]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Center tile (1,1) has all 4 neighbors → bitmask 15 → local 15 → gid 16
    assert_eq!(layer.data[4].gid(), 16); // first_gid(1) + local(15) = 16
}

#[test]
fn bitmask_8bit_diagonal_rule() {
    // 3x3 grid with 8-bit stone tiles
    // All cells filled with stone. Center should get full bitmask.
    let ts = make_8bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 3);

    let s = TileId::from_gid(100); // stone local 0

    #[rustfmt::skip]
    let mut layer = make_tile_layer("ground", vec![
        s, s, s,
        s, s, s,
        s, s, s,
    ]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Center (1,1): all 8 neighbors present and all cardinals connected
    // → all diagonals count → bitmask = 1+2+4+8+16+32+64+128 = 255
    // → local 255 → gid 100+255 = 355
    assert_eq!(layer.data[4].gid(), 355);
}

#[test]
fn bitmask_8bit_diagonal_without_cardinal() {
    // Only diagonal neighbors present, no cardinals → diagonals should NOT count
    let ts = make_8bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 3);

    let s = TileId::from_gid(100); // stone
    let e = TileId::EMPTY;

    // Checkerboard pattern: diagonals present, cardinals empty
    #[rustfmt::skip]
    let mut layer = make_tile_layer("ground", vec![
        s, e, s,
        e, s, e,
        s, e, s,
    ]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Center (1,1): no cardinals → no diagonals count → bitmask 0 → local 0 → gid 100
    assert_eq!(layer.data[4].gid(), 100);
}

#[test]
fn resolve_simple_4bit() {
    // 3x3 with grass in cross pattern:
    //   . G .
    //   G G G
    //   . G .
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 3);

    let e = TileId::EMPTY;
    let g = TileId::from_gid(1);

    #[rustfmt::skip]
    let mut layer = make_tile_layer("ground", vec![
        e, g, e,
        g, g, g,
        e, g, e,
    ]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Center (1,1): N=1, E=2, S=4, W=8 → bitmask 15 → local 15 → gid 16
    assert_eq!(layer.data[4].gid(), 16);

    // Top center (1,0): N=0, E=0, S=1 (center), W=0 → bitmask 4 → local 4 → gid 5
    assert_eq!(layer.data[1].gid(), 5);

    // Left center (0,1): N=0, E=1 (center), S=0, W=0 → bitmask 2 → local 2 → gid 3
    assert_eq!(layer.data[3].gid(), 3);

    // Right center (2,1): N=0, E=0, S=0, W=1 (center) → bitmask 8 → local 8 → gid 9
    assert_eq!(layer.data[5].gid(), 9);

    // Bottom center (1,2): N=1 (center), E=0, S=0, W=0 → bitmask 1 → local 1 → gid 2
    assert_eq!(layer.data[7].gid(), 2);
}

#[test]
fn resolve_preserves_flags() {
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(1, 1);

    // Single tile with horizontal flip flag set
    let flagged = TileId::new(1, true, false, false);
    assert!(flagged.flip_horizontal());

    let mut layer = make_tile_layer("ground", vec![flagged]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // After resolve: bitmask 0 → local 0 → gid 1, flags preserved
    let result = layer.data[0];
    assert_eq!(result.gid(), 1);
    assert!(result.flip_horizontal());
    assert!(!result.flip_vertical());
}

#[test]
fn topology_wrap_x() {
    // 3x1 row of grass tiles, wrap_x topology
    // Tile at x=0 should connect to tile at x=2 (via wrap)
    let ts = make_4bit_tileset();
    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_grid_with_topology(3, 1, Topology::WrapX);

    let g = TileId::from_gid(1);

    let mut layer = make_tile_layer("ground", vec![g, g, g]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Each tile: E and W connected (wrap), N and S not (bounded in y, no rows)
    // bitmask: E=2 + W=8 = 10 → local 10 → gid 11
    assert_eq!(layer.data[0].gid(), 11);
    assert_eq!(layer.data[1].gid(), 11);
    assert_eq!(layer.data[2].gid(), 11);
}

#[test]
fn fallback_to_isolated() {
    // Create a tileset that only has bitmask 0 and bitmask 15 variants
    let mut tiles = HashMap::new();
    tiles.insert(
        "0".to_string(),
        TileData {
            properties: HashMap::new(),
            auto_tile: Some(AutoTile {
                group: "sparse".to_string(),
                rule: AutoTileRule::Bitmask4bit,
                bitmask: 0,
            }),
            extra: HashMap::new(),
        },
    );
    tiles.insert(
        "1".to_string(),
        TileData {
            properties: HashMap::new(),
            auto_tile: Some(AutoTile {
                group: "sparse".to_string(),
                rule: AutoTileRule::Bitmask4bit,
                bitmask: 15,
            }),
            extra: HashMap::new(),
        },
    );

    let ts = Tileset {
        name: "sparse_set".to_string(),
        tile_width: 16,
        tile_height: 16,
        image: "sparse.png".to_string(),
        image_width: 256,
        image_height: 256,
        columns: 16,
        tile_count: 16,
        margin: 0,
        spacing: 0,
        first_gid: 1,
        tiles,
        extra: HashMap::new(),
    };

    let entries = vec![TilesetEntry::Inline(ts)];
    let index = build_autotile_index(&entries);
    let grid = make_bounded_grid(3, 1);

    // Three tiles in a row → middle has E=2, W=8 → bitmask 10 → no match → fallback 0
    let g = TileId::from_gid(1);
    let mut layer = make_tile_layer("ground", vec![g, g, g]);

    resolve_autotiles(&mut layer, &grid, &index, &entries);

    // Middle tile: bitmask 10 not in index, falls back to bitmask 0 → local 0 → gid 1
    assert_eq!(layer.data[1].gid(), 1);
}
