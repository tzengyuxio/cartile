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
    assert_eq!(
        map.grid.projection.projection_type,
        ProjectionType::Orthogonal
    );
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

#[test]
fn convert_object_layer() {
    let tiled = load_tiled("with_objects.json");
    let (map, _) = convert_tiled_map(&tiled, "objects", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.layers.len(), 2);
    match &map.layers[1] {
        Layer::Object(ol) => {
            assert_eq!(ol.objects.len(), 5);
            assert_eq!(
                ol.objects[0].shape,
                cartile_format::types::object::Shape::Point
            );
            assert_eq!(ol.objects[0].id, 1);
            assert_eq!(ol.objects[0].name, "spawn");
            assert_eq!(
                ol.objects[1].shape,
                cartile_format::types::object::Shape::Rect
            );
            assert_eq!(ol.objects[1].rotation, 45.0);
            assert_eq!(
                ol.objects[2].shape,
                cartile_format::types::object::Shape::Ellipse
            );
            assert_eq!(
                ol.objects[3].shape,
                cartile_format::types::object::Shape::Polyline
            );
            assert_eq!(ol.objects[3].points.as_ref().unwrap().len(), 3);
            assert_eq!(
                ol.objects[4].shape,
                cartile_format::types::object::Shape::Polygon
            );
            assert_eq!(ol.objects[4].points.as_ref().unwrap().len(), 4);
        }
        _ => panic!("expected object layer"),
    }
    map.validate().unwrap();
}

#[test]
fn convert_hex_map() {
    let tiled = load_tiled("hexagonal.json");
    let (map, _) = convert_tiled_map(&tiled, "hex", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.grid.grid_type, GridType::Hexagonal);
    assert_eq!(map.grid.orientation, Some(HexOrientation::PointyTop));
    assert!(map.grid.stagger.is_some());
    let stagger = map.grid.stagger.unwrap();
    assert_eq!(stagger.axis, StaggerAxis::Y);
    assert_eq!(stagger.index, StaggerIndex::Odd);
    assert_eq!(
        map.grid.projection.projection_type,
        ProjectionType::Orthogonal
    );

    map.validate().unwrap();
}

#[test]
fn convert_external_tileset_inline() {
    let tiled = load_tiled("external_tileset.json");
    let (map, _) = convert_tiled_map(&tiled, "ext", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.first_gid, 1);
            assert_eq!(ts.tile_count, 16);
        }
        _ => panic!("expected inline tileset (external should be resolved)"),
    }
    map.validate().unwrap();
}

#[test]
fn convert_infinite_map_errors() {
    let json = r#"{
        "type": "map", "version": "1.10", "tiledversion": "1.11.2",
        "orientation": "orthogonal", "renderorder": "right-down",
        "width": 10, "height": 10, "tilewidth": 16, "tileheight": 16,
        "infinite": true, "layers": [], "tilesets": []
    }"#;
    let tiled: cartile_cli::tiled::types::TiledMap = serde_json::from_str(json).unwrap();
    let result = convert_tiled_map(&tiled, "test", Path::new("."), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("infinite"));
}

#[test]
fn convert_external_tileset_to_file() {
    let tiled = load_tiled("external_tileset.json");
    let temp_dir = std::env::temp_dir().join("cartile_test_external_ts");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let (map, _warnings) = convert_tiled_map(
        &tiled,
        "ext",
        Path::new("tests/fixtures"),
        Some(temp_dir.as_path()),
    )
    .unwrap();

    // Should have an ExternalRef entry
    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::ExternalRef(r) => {
            assert_eq!(r.ref_path, "./terrain.cartile-ts");
            assert_eq!(r.first_gid, 1);
        }
        _ => panic!("expected external ref tileset"),
    }

    // Verify the .cartile-ts file was written
    let ts_path = temp_dir.join("terrain.cartile-ts");
    assert!(ts_path.exists(), "tileset file should exist on disk");

    // Verify it can be parsed as TilesetFile
    let ts_json = std::fs::read_to_string(&ts_path).unwrap();
    let ts_file: TilesetFile = serde_json::from_str(&ts_json).unwrap();
    assert_eq!(ts_file.cartile, "0.1.0");
    assert_eq!(ts_file.file_type, "tileset");
    assert_eq!(ts_file.tileset.name, "terrain");
    assert_eq!(ts_file.tileset.tile_count, 16);
    // Standalone file should not have first_gid
    assert_eq!(ts_file.tileset.first_gid, 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
