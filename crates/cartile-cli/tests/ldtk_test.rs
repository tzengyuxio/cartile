use cartile_cli::ldtk::convert::{convert_ldtk_project, is_ldtk_json};
use cartile_cli::ldtk::types::LdtkRoot;
use cartile_format::*;

fn load_ldtk(filename: &str) -> LdtkRoot {
    let path = format!("tests/fixtures/{filename}");
    let json = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn detect_ldtk_json() {
    let ldtk_json = std::fs::read_to_string("tests/fixtures/ldtk_simple.json").unwrap();
    assert!(is_ldtk_json(&ldtk_json));

    let tiled_json = std::fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    assert!(!is_ldtk_json(&tiled_json));
}

#[test]
fn convert_ldtk_tile_layer() {
    let root = load_ldtk("ldtk_simple.json");
    let (map, warnings) = convert_ldtk_project(&root, None).unwrap();

    // No multi-level warning for single-level project
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

    assert_eq!(map.cartile, "0.1.0");
    assert_eq!(map.name, "Level_0");
    assert_eq!(map.grid.grid_type, GridType::Square);
    assert_eq!(
        map.grid.projection.projection_type,
        ProjectionType::Orthogonal
    );
    assert_eq!(map.grid.width, 3);
    assert_eq!(map.grid.height, 2);
    assert_eq!(map.grid.tile_width, 16);
    assert_eq!(map.grid.tile_height, 16);

    // One tileset
    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "Terrain");
            assert_eq!(ts.first_gid, 1);
            assert_eq!(ts.tile_count, 16);
            assert_eq!(ts.columns, 4);
        }
        _ => panic!("expected inline tileset"),
    }

    // Two layers: Objects (entities, reversed to bottom) then Ground (tiles, on top)
    // LDtk order is reversed: Ground was first (top), Objects was second (bottom)
    // After reversal: Objects first, Ground second
    assert_eq!(map.layers.len(), 2);

    // First layer should be the Objects (entity) layer
    match &map.layers[0] {
        Layer::Object(ol) => {
            assert_eq!(ol.name, "Objects");
            assert_eq!(ol.objects.len(), 2);
            assert_eq!(ol.objects[0].name, "Player");
            assert_eq!(ol.objects[0].x, 8.0);
            assert_eq!(ol.objects[0].y, 8.0);
            assert_eq!(ol.objects[0].shape, Shape::Rect);
            assert_eq!(ol.objects[1].name, "Coin");
            assert_eq!(ol.objects[1].shape, Shape::Point);
        }
        _ => panic!("expected object layer"),
    }

    // Second layer should be the Ground (tile) layer
    match &map.layers[1] {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "Ground");
            assert_eq!(tl.data.len(), 6);
            // tile_id 0 + first_gid 1 = gid 1
            assert_eq!(tl.data[0].gid(), 1);
            // tile_id 1 + first_gid 1 = gid 2
            assert_eq!(tl.data[1].gid(), 2);
            // tile_id 2 + first_gid 1 = gid 3, flip-x
            assert_eq!(tl.data[2].gid(), 3);
            assert!(tl.data[2].flip_horizontal());
            assert!(!tl.data[2].flip_vertical());
            // tile_id 4 + first_gid 1 = gid 5, flip-y
            assert_eq!(tl.data[4].gid(), 5);
            assert!(!tl.data[4].flip_horizontal());
            assert!(tl.data[4].flip_vertical());
        }
        _ => panic!("expected tile layer"),
    }

    map.validate().unwrap();
}

#[test]
fn convert_ldtk_intgrid_layer() {
    let root = load_ldtk("ldtk_intgrid.json");
    let (map, _) = convert_ldtk_project(&root, None).unwrap();

    assert_eq!(map.name, "IntGrid_Level");
    assert_eq!(map.layers.len(), 1);

    match &map.layers[0] {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "Collision");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0].gid(), 1);
            assert!(tl.data[1].is_empty());
            assert_eq!(tl.data[2].gid(), 1);
            assert!(tl.data[3].is_empty());
            assert_eq!(tl.data[4].gid(), 2);
            assert!(tl.data[5].is_empty());
        }
        _ => panic!("expected tile layer"),
    }

    map.validate().unwrap();
}

#[test]
fn convert_ldtk_no_levels_errors() {
    let json = r#"{
        "__header__": {"app": "LDtk"},
        "jsonVersion": "1.5.3",
        "defaultGridSize": 16,
        "defs": {"tilesets": []},
        "worlds": [],
        "levels": []
    }"#;
    let root: LdtkRoot = serde_json::from_str(json).unwrap();
    let result = convert_ldtk_project(&root, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no levels"));
}
