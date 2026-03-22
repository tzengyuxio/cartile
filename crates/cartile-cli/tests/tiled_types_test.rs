use std::fs;

#[test]
fn parse_orthogonal_map() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    assert_eq!(map.orientation, "orthogonal");
    assert_eq!(map.width, 3);
    assert_eq!(map.height, 2);
    assert_eq!(map.tilewidth, 32);
    assert!(!map.infinite);
    assert_eq!(map.layers.len(), 1);
    assert_eq!(map.tilesets.len(), 1);
}

#[test]
fn parse_tile_layer() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    match &map.layers[0] {
        cartile_cli::tiled::types::TiledLayer::TileLayer(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0], 1);
            assert!(tl.visible);
        }
        _ => panic!("expected tile layer"),
    }
}

#[test]
fn parse_embedded_tileset() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    match &map.tilesets[0] {
        cartile_cli::tiled::types::TiledTilesetEntry::Embedded(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.firstgid, 1);
            assert_eq!(ts.tilecount, 16);
        }
        _ => panic!("expected embedded tileset"),
    }
}
