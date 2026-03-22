use cartile_format::types::layer::*;
use cartile_format::types::object::*;

#[test]
fn tile_layer_serde() {
    let json = r#"{
        "type": "tile",
        "name": "ground",
        "elevation": 0,
        "encoding": "dense",
        "data": [1, 0, 2, 2147483649]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 4);
            assert!(tl.data[1].is_empty());
            assert_eq!(tl.data[3].gid(), 1);
            assert!(tl.data[3].flip_horizontal());
            assert!(tl.visible);
            assert_eq!(tl.opacity, 1.0);
        }
        _ => panic!("expected tile layer"),
    }
}

#[test]
fn object_layer_serde() {
    let json = r#"{
        "type": "object",
        "name": "entities",
        "objects": [
            { "id": 1, "name": "spawn", "x": 128.0, "y": 64.0, "shape": "point" },
            { "id": 2, "name": "zone", "x": 0.0, "y": 0.0, "width": 100.0, "height": 50.0,
              "shape": "rect", "rotation": 45.0,
              "properties": { "type": { "type": "string", "value": "trigger" } } },
            { "id": 3, "name": "path", "x": 10.0, "y": 20.0, "shape": "polyline",
              "points": [{"x": 0, "y": 0}, {"x": 32, "y": 16}] }
        ]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Object(ol) => {
            assert_eq!(ol.objects.len(), 3);
            assert_eq!(ol.objects[0].shape, Shape::Point);
            assert_eq!(ol.objects[1].rotation, 45.0);
            assert_eq!(ol.objects[2].shape, Shape::Polyline);
            assert_eq!(ol.objects[2].points.as_ref().unwrap().len(), 2);
        }
        _ => panic!("expected object layer"),
    }
}

#[test]
fn heightmap_layer_serde() {
    let json = r#"{
        "type": "heightmap",
        "name": "terrain_height",
        "data": [0, 1, 2, -1, 0, 3]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Heightmap(hm) => {
            assert_eq!(hm.name, "terrain_height");
            assert_eq!(hm.data, vec![0, 1, 2, -1, 0, 3]);
        }
        _ => panic!("expected heightmap layer"),
    }
}
