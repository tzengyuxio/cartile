use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of named properties attached to any cartile object.
pub type Properties = HashMap<String, Property>;

/// A typed property value. The `type` field discriminates how `value` should
/// be interpreted, allowing the editor and runtime to validate and display
/// properties correctly without relying on JSON type inference alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: PropertyType,
    pub value: serde_json::Value,
}

/// Supported property types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Bool,
    String,
    Int,
    Float,
    Color,
    File,
}
