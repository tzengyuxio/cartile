use cartile_cli::tiled::convert;

#[test]
fn gid_empty() {
    assert_eq!(convert::convert_gid(0), (0, vec![]));
}

#[test]
fn gid_simple() {
    assert_eq!(convert::convert_gid(42), (42, vec![]));
}

#[test]
fn gid_horizontal_flip() {
    let raw = 0x80000001u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    assert_eq!(cartile_raw, 0x80000001);
    assert!(warnings.is_empty());
}

#[test]
fn gid_hex_rotation_flag_cleared() {
    let raw = 0x10000005u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    assert_eq!(cartile_raw, 5);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("hex rotation"));
}

#[test]
fn gid_all_flags() {
    let raw = 0xF0000007u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    assert_eq!(cartile_raw, 0xE0000007);
    assert_eq!(warnings.len(), 1);
}

#[test]
fn convert_color_aarrggbb_to_rrggbbaa() {
    assert_eq!(convert::convert_color("#FF336699"), "#336699FF");
}

#[test]
fn convert_color_rrggbb_to_rrggbbff() {
    assert_eq!(convert::convert_color("#336699"), "#336699FF");
}

#[test]
fn convert_property_string() {
    use cartile_format::PropertyType;
    let tp = cartile_cli::tiled::types::TiledProperty {
        name: "key".to_string(),
        property_type: "string".to_string(),
        value: serde_json::json!("hello"),
    };
    let (name, prop, warnings) = convert::convert_property(&tp);
    assert_eq!(name, "key");
    assert_eq!(prop.property_type, PropertyType::String);
    assert!(warnings.is_empty());
}

#[test]
fn convert_property_object_type_skipped() {
    let tp = cartile_cli::tiled::types::TiledProperty {
        name: "target".to_string(),
        property_type: "object".to_string(),
        value: serde_json::json!(42),
    };
    let (_, _, warnings) = convert::convert_property(&tp);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("unsupported"));
}
