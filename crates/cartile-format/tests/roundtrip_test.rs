use cartile_format::*;

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
