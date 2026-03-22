use cartile_format::{Properties, Property, PropertyType};

use super::types::TiledProperty;

const TILED_GID_MASK: u32 = 0x0FFF_FFFF;
const HEX_ROTATION_BIT: u32 = 0x1000_0000;
const CARTILE_FLAG_MASK: u32 = 0xE000_0000;

/// Convert a Tiled GID (with 4 flag bits) to a cartile raw TileId value (3 flag bits).
/// Returns (cartile_raw, warnings).
pub fn convert_gid(tiled_raw: u32) -> (u32, Vec<String>) {
    if tiled_raw == 0 {
        return (0, vec![]);
    }
    let mut warnings = vec![];
    let tiled_flags = tiled_raw & 0xF000_0000;
    let tiled_gid = tiled_raw & TILED_GID_MASK;

    if tiled_flags & HEX_ROTATION_BIT != 0 {
        warnings.push("tile has hex rotation flag (bit 28) — cleared".to_string());
    }

    let cartile_flags = tiled_flags & CARTILE_FLAG_MASK;
    (cartile_flags | tiled_gid, warnings)
}

/// Convert a Tiled color string to cartile format.
/// Tiled: `#AARRGGBB` or `#RRGGBB`
/// Cartile: `#RRGGBBAA`
pub fn convert_color(tiled_color: &str) -> String {
    let hex = tiled_color.trim_start_matches('#');
    match hex.len() {
        8 => {
            let aa = &hex[0..2];
            let rrggbb = &hex[2..8];
            format!("#{rrggbb}{aa}")
        }
        6 => format!("#{hex}FF"),
        _ => tiled_color.to_string(),
    }
}

/// Convert a single Tiled property to cartile format.
/// Returns (name, Property, warnings). If the property type is unsupported,
/// warnings will be non-empty.
pub fn convert_property(tp: &TiledProperty) -> (String, Property, Vec<String>) {
    let mut warnings = vec![];
    let (prop_type, value) = match tp.property_type.as_str() {
        "string" => (PropertyType::String, tp.value.clone()),
        "int" => (PropertyType::Int, tp.value.clone()),
        "float" => (PropertyType::Float, tp.value.clone()),
        "bool" => (PropertyType::Bool, tp.value.clone()),
        "file" => (PropertyType::File, tp.value.clone()),
        "color" => {
            let converted = tp
                .value
                .as_str()
                .map(|s| serde_json::Value::String(convert_color(s)))
                .unwrap_or_else(|| tp.value.clone());
            (PropertyType::Color, converted)
        }
        other => {
            warnings.push(format!(
                "unsupported property type '{other}' for '{}' — skipped",
                tp.name
            ));
            (PropertyType::String, tp.value.clone())
        }
    };
    (tp.name.clone(), Property { property_type: prop_type, value }, warnings)
}

/// Convert a Vec of Tiled properties to a cartile Properties map.
/// Unsupported types are skipped.
pub fn convert_properties(tiled_props: &[TiledProperty]) -> (Properties, Vec<String>) {
    let mut props = Properties::new();
    let mut all_warnings = vec![];
    for tp in tiled_props {
        let (name, prop, warnings) = convert_property(tp);
        let is_unsupported = !warnings.is_empty();
        all_warnings.extend(warnings);
        if !is_unsupported {
            props.insert(name, prop);
        }
    }
    (props, all_warnings)
}
