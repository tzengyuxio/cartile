use cartile_cli::tiled::convert::convert_tiled_map;
use cartile_cli::tiled::export::export_to_tiled;
use cartile_cli::tiled::types::TiledMap;
use std::path::Path;

fn load_tiled(filename: &str) -> TiledMap {
    let path = format!("tests/fixtures/{filename}");
    let json = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn roundtrip_orthogonal() {
    let original = load_tiled("orthogonal.json");
    let (cartile_map, _) =
        convert_tiled_map(&original, "orthogonal", Path::new("tests/fixtures"), None).unwrap();

    let (exported, warnings) = export_to_tiled(&cartile_map).unwrap();
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

    assert_eq!(exported.orientation, "orthogonal");
    assert_eq!(exported.width, original.width);
    assert_eq!(exported.height, original.height);
    assert_eq!(exported.tilewidth, original.tilewidth);
    assert_eq!(exported.tileheight, original.tileheight);
    assert!(!exported.infinite);
    assert_eq!(exported.renderorder.as_deref(), Some("right-down"));

    // Tileset preserved
    assert_eq!(exported.tilesets.len(), 1);

    // Layer preserved
    assert_eq!(exported.layers.len(), 1);
    match &exported.layers[0] {
        cartile_cli::tiled::types::TiledLayer::TileLayer(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0], 1);
            assert_eq!(tl.data[5], 3);
        }
        _ => panic!("expected tile layer"),
    }

    // Verify the exported map can be serialized and parsed back
    let json = serde_json::to_string_pretty(&exported).unwrap();
    let reparsed: TiledMap = serde_json::from_str(&json).unwrap();
    assert_eq!(reparsed.orientation, "orthogonal");
    assert_eq!(reparsed.width, 3);
}

#[test]
fn roundtrip_hexagonal() {
    let original = load_tiled("hexagonal.json");
    let (cartile_map, _) =
        convert_tiled_map(&original, "hex", Path::new("tests/fixtures"), None).unwrap();

    let (exported, _) = export_to_tiled(&cartile_map).unwrap();

    assert_eq!(exported.orientation, "hexagonal");
    assert_eq!(exported.staggeraxis.as_deref(), Some("y"));
    assert_eq!(exported.staggerindex.as_deref(), Some("odd"));
}

#[test]
fn export_object_layer_preserved() {
    let original = load_tiled("with_objects.json");
    let (cartile_map, _) =
        convert_tiled_map(&original, "objects", Path::new("tests/fixtures"), None).unwrap();

    let (exported, _) = export_to_tiled(&cartile_map).unwrap();

    assert_eq!(exported.layers.len(), 2);
    match &exported.layers[1] {
        cartile_cli::tiled::types::TiledLayer::ObjectGroup(ol) => {
            assert_eq!(ol.objects.len(), 5);

            // Point
            assert_eq!(ol.objects[0].name, "spawn");
            assert!(ol.objects[0].point);
            assert_eq!(ol.objects[0].id, 1);

            // Rect
            assert_eq!(ol.objects[1].name, "zone");
            assert_eq!(ol.objects[1].rotation, 45.0);
            assert!(!ol.objects[1].point);
            assert!(!ol.objects[1].ellipse);
            assert!(ol.objects[1].polygon.is_none());
            assert!(ol.objects[1].polyline.is_none());

            // Ellipse
            assert!(ol.objects[2].ellipse);

            // Polyline
            assert!(ol.objects[3].polyline.is_some());
            assert_eq!(ol.objects[3].polyline.as_ref().unwrap().len(), 3);

            // Polygon
            assert!(ol.objects[4].polygon.is_some());
            assert_eq!(ol.objects[4].polygon.as_ref().unwrap().len(), 4);
        }
        _ => panic!("expected object group layer"),
    }
}
