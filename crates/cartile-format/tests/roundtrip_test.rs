use cartile_format::*;
use serde_json::Value;

#[test]
fn minimal_map_roundtrip() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [
            {
                "type": "tile",
                "name": "ground",
                "data": [0, 0, 0, 0]
            }
        ]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    assert_eq!(map.cartile, "0.1.0");
    assert_eq!(map.name, "test");
    assert_eq!(map.grid.width, 2);

    let serialized = serde_json::to_string_pretty(&map).unwrap();
    let back: CartileMap = serde_json::from_str(&serialized).unwrap();
    assert_eq!(map, back);
}

#[test]
fn unknown_fields_preserved() {
    let json = r#"{
        "cartile": "0.2.0",
        "type": "map",
        "name": "future",
        "future_field": "hello",
        "grid": {
            "type": "square",
            "width": 1,
            "height": 1,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": []
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let serialized = serde_json::to_string(&map).unwrap();
    assert!(serialized.contains("future_field"));
    assert!(serialized.contains("hello"));
}

#[test]
fn read_from_file() {
    let map = CartileMap::from_file("tests/fixtures/minimal_map.cartile").unwrap();
    assert_eq!(map.name, "minimal");
}

#[test]
fn write_to_file() {
    let map = CartileMap::from_file("tests/fixtures/minimal_map.cartile").unwrap();
    let tmp = std::env::temp_dir().join("cartile_test_output.cartile");
    map.to_file(&tmp).unwrap();
    let back = CartileMap::from_file(&tmp).unwrap();
    assert_eq!(map, back);
    std::fs::remove_file(tmp).ok();
}

#[test]
fn srpg_map_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/srpg_map.cartile").unwrap();
    assert_eq!(map.name, "tutorial_battlefield");
    assert_eq!(map.grid.height_mode, HeightMode::Stepped);
    assert_eq!(map.tilesets.len(), 1);
    assert_eq!(map.layers.len(), 3);
    assert!(map.validate().is_ok());

    let json = map.to_json_pretty().unwrap();
    let back: CartileMap = serde_json::from_str(&json).unwrap();
    assert_eq!(map, back);
}

#[test]
fn hex_map_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/hex_map.cartile").unwrap();
    assert_eq!(map.grid.grid_type, GridType::Hexagonal);
    assert!(map.validate().is_ok());
}

#[test]
fn vertex_height_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/vertex_height.cartile").unwrap();
    assert_eq!(map.grid.height_mode, HeightMode::Vertex);
    assert!(map.validate().is_ok());
}

#[test]
fn external_ref_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/external_ref.cartile").unwrap();
    match &map.tilesets[0] {
        TilesetEntry::ExternalRef(r) => {
            assert_eq!(r.ref_path, "./terrain.cartile-ts");
            assert_eq!(r.first_gid, 1);
        }
        _ => panic!("expected external ref"),
    }
}

#[test]
fn tileset_file_roundtrip() {
    let content = std::fs::read_to_string("tests/fixtures/terrain.cartile-ts").unwrap();
    let tsf: TilesetFile = serde_json::from_str(&content).unwrap();
    assert_eq!(tsf.tileset.name, "terrain");
    assert_eq!(tsf.tileset.tile_count, 16);
}

#[test]
fn generate_schema() {
    let schema = cartile_format::generate_map_schema();
    let parsed: Value = serde_json::from_str(&schema).unwrap();
    assert_eq!(parsed["type"], "object");
    assert!(parsed["properties"]["cartile"].is_object());
    assert!(parsed["properties"]["grid"].is_object());
    assert!(parsed["properties"]["layers"].is_object());
}
