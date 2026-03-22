use cartile_format::types::tileset::*;

#[test]
fn inline_tileset_serde() {
    let json = r#"{
        "name": "terrain",
        "tile_width": 16,
        "tile_height": 16,
        "image": "assets/terrain.png",
        "image_width": 256,
        "image_height": 256,
        "columns": 16,
        "tile_count": 256,
        "first_gid": 1,
        "tiles": {
            "5": {
                "properties": {
                    "walkable": { "type": "bool", "value": false }
                },
                "auto_tile": {
                    "group": "water",
                    "rule": "bitmask_4bit",
                    "bitmask": 0
                }
            }
        }
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.tile_count, 256);
            assert_eq!(ts.first_gid, 1);
            let tile5 = ts.tiles.get("5").unwrap();
            let at = tile5.auto_tile.as_ref().unwrap();
            assert_eq!(at.group, "water");
            assert_eq!(at.rule, AutoTileRule::Bitmask4bit);
            assert_eq!(at.bitmask, 0);
        }
        _ => panic!("expected inline tileset"),
    }
}

#[test]
fn external_ref_serde() {
    let json = r#"{
        "$ref": "./tilesets/characters.cartile-ts",
        "first_gid": 257
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::ExternalRef(r) => {
            assert_eq!(r.ref_path, "./tilesets/characters.cartile-ts");
            assert_eq!(r.first_gid, 257);
        }
        _ => panic!("expected external ref"),
    }
}

#[test]
fn optional_fields_default() {
    let json = r#"{
        "name": "test",
        "tile_width": 16,
        "tile_height": 16,
        "image": "test.png",
        "image_width": 64,
        "image_height": 64,
        "columns": 4,
        "tile_count": 16,
        "first_gid": 1
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.margin, 0);
            assert_eq!(ts.spacing, 0);
            assert!(ts.tiles.is_empty());
        }
        _ => panic!("expected inline"),
    }
}
