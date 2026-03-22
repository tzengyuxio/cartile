use cartile_cli::tiled::convert::convert_tiled_map;
use cartile_cli::tiled::types::TiledMap;
use cartile_format::*;
use std::path::Path;

fn load_tiled(filename: &str) -> TiledMap {
    let path = format!("tests/fixtures/{filename}");
    let json = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn convert_orthogonal_map() {
    let tiled = load_tiled("orthogonal.json");
    let (map, warnings) =
        convert_tiled_map(&tiled, "orthogonal", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.cartile, "0.1.0");
    assert_eq!(map.map_type, "map");
    assert_eq!(map.name, "orthogonal");
    assert_eq!(map.grid.grid_type, GridType::Square);
    assert_eq!(map.grid.projection.projection_type, ProjectionType::Orthogonal);
    assert_eq!(map.grid.width, 3);
    assert_eq!(map.grid.height, 2);
    assert_eq!(map.grid.topology, Topology::Bounded);
    assert_eq!(map.grid.height_mode, HeightMode::None);
    assert!(map.grid.stagger.is_none());
    assert!(map.grid.orientation.is_none());

    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.first_gid, 1);
            assert_eq!(ts.tile_count, 16);
        }
        _ => panic!("expected inline tileset"),
    }

    assert_eq!(map.layers.len(), 1);
    match &map.layers[0] {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0].gid(), 1);
            assert!(tl.visible);
        }
        _ => panic!("expected tile layer"),
    }

    // Warn count: renderorder "right-down" is default, so no warning expected
    let _ = warnings;

    map.validate().unwrap();
}
