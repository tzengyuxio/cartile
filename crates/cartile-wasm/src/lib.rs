use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Core logic (returns Result<T, String>, testable on native targets)
// ---------------------------------------------------------------------------

/// Parse and validate a cartile map JSON string.
/// Returns the parsed map as a pretty-printed JSON string, or an error message.
pub fn parse_cartile_map_inner(json: &str) -> Result<String, String> {
    let map = cartile_format::CartileMap::from_json(json).map_err(|e| e.to_string())?;
    let output = map.to_json_pretty().map_err(|e| e.to_string())?;
    Ok(output)
}

/// Validate a cartile map JSON string.
/// Returns Ok(()) on success, or an error message.
pub fn validate_cartile_map_inner(json: &str) -> Result<(), String> {
    let map = cartile_format::CartileMap::from_json(json).map_err(|e| e.to_string())?;
    map.validate().map_err(|e| e.to_string())?;
    Ok(())
}

/// Convert a Tiled JSON export to cartile format.
/// Returns a JSON object string: { "cartile_json": "...", "warnings": ["..."] }
pub fn convert_tiled_json_inner(tiled_json: &str, map_name: &str) -> Result<String, String> {
    let tiled_map: cartile_cli::tiled::types::TiledMap =
        serde_json::from_str(tiled_json).map_err(|e| format!("invalid Tiled JSON: {e}"))?;

    let (map, warnings) = cartile_cli::tiled::convert::convert_tiled_map(
        &tiled_map,
        map_name,
        std::path::Path::new("."),
        None,
    )
    .map_err(|e| e.to_string())?;

    map.validate()
        .map_err(|e| format!("conversion produced invalid output: {e}"))?;

    let cartile_json = map.to_json_pretty().map_err(|e| e.to_string())?;

    let result = serde_json::json!({
        "cartile_json": cartile_json,
        "warnings": warnings,
    });
    Ok(result.to_string())
}

/// Resolve auto-tiles in a cartile map JSON string.
/// Returns the map JSON with auto-tiles resolved.
pub fn resolve_autotiles_inner(map_json: &str) -> Result<String, String> {
    let mut map: cartile_format::CartileMap =
        serde_json::from_str(map_json).map_err(|e| e.to_string())?;

    let index = cartile_format::build_autotile_index(&map.tilesets);

    for layer in &mut map.layers {
        if let cartile_format::Layer::Tile(tile_layer) = layer {
            cartile_format::resolve_autotiles(tile_layer, &map.grid, &index, &map.tilesets);
        }
    }

    let output = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    Ok(output)
}

// ---------------------------------------------------------------------------
// WASM bindings (thin wrappers that convert String errors to JsError)
// ---------------------------------------------------------------------------

/// Parse and validate a cartile map JSON string.
/// Returns the parsed map as a pretty-printed JSON string, or an error.
#[wasm_bindgen(js_name = "parseCartileMap")]
pub fn parse_cartile_map(json: &str) -> Result<String, JsError> {
    parse_cartile_map_inner(json).map_err(|e| JsError::new(&e))
}

/// Validate a cartile map JSON string.
/// Returns null/undefined on success, or throws an error with the validation message.
#[wasm_bindgen(js_name = "validateCartileMap")]
pub fn validate_cartile_map(json: &str) -> Result<(), JsError> {
    validate_cartile_map_inner(json).map_err(|e| JsError::new(&e))
}

/// Convert a Tiled JSON export to cartile format.
/// Returns a JSON object string: { "cartile_json": "...", "warnings": ["..."] }
#[wasm_bindgen(js_name = "convertTiledJson")]
pub fn convert_tiled_json(tiled_json: &str, map_name: &str) -> Result<String, JsError> {
    convert_tiled_json_inner(tiled_json, map_name).map_err(|e| JsError::new(&e))
}

/// Resolve auto-tiles in a cartile map JSON string.
/// Returns the map JSON with auto-tiles resolved.
#[wasm_bindgen(js_name = "resolveAutotiles")]
pub fn resolve_autotiles(map_json: &str) -> Result<String, JsError> {
    resolve_autotiles_inner(map_json).map_err(|e| JsError::new(&e))
}
