// These tests run as native Rust (not WASM) to verify the JSON boundary logic.
// They call the `_inner` variants which return Result<T, String> instead of
// Result<T, JsError>, since JsError cannot be constructed on non-WASM targets.

#[test]
fn parse_valid_map() {
    let json =
        std::fs::read_to_string("../cartile-format/tests/fixtures/minimal_map.cartile").unwrap();
    let result = cartile_wasm::parse_cartile_map_inner(&json);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("\"cartile\""));
    assert!(output.contains("\"0.1.0\""));
}

#[test]
fn parse_invalid_json() {
    let result = cartile_wasm::parse_cartile_map_inner("not json");
    assert!(result.is_err());
}

#[test]
fn validate_valid_map() {
    let json =
        std::fs::read_to_string("../cartile-format/tests/fixtures/minimal_map.cartile").unwrap();
    let result = cartile_wasm::validate_cartile_map_inner(&json);
    assert!(result.is_ok());
}

#[test]
fn validate_invalid_map() {
    // Map with wrong data length
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "bad",
        "grid": {
            "type": "square",
            "width": 2, "height": 2,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "data": [0, 0, 0] }]
    }"#;
    let result = cartile_wasm::validate_cartile_map_inner(json);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("data length"));
}

#[test]
fn convert_tiled_json_basic() {
    let tiled = std::fs::read_to_string("../cartile-cli/tests/fixtures/orthogonal.json").unwrap();
    let result = cartile_wasm::convert_tiled_json_inner(&tiled, "test_map");
    assert!(result.is_ok());
    let output = result.unwrap();
    // Parse the result JSON wrapper
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["cartile_json"].is_string());
    assert!(parsed["warnings"].is_array());
    // The inner cartile_json should be valid
    let inner = parsed["cartile_json"].as_str().unwrap();
    assert!(inner.contains("\"cartile\""));
}

#[test]
fn convert_invalid_tiled_json() {
    let result = cartile_wasm::convert_tiled_json_inner("{}", "test");
    assert!(result.is_err());
}

#[test]
fn resolve_autotiles_passthrough() {
    // Map without any auto-tile groups — should pass through unchanged
    let json =
        std::fs::read_to_string("../cartile-format/tests/fixtures/minimal_map.cartile").unwrap();
    let result = cartile_wasm::resolve_autotiles_inner(&json);
    assert!(result.is_ok());
}
