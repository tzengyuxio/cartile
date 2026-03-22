use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

use super::property::Properties;

/// A tileset entry in a map file — either inline or an external reference.
#[derive(Debug, Clone, PartialEq)]
pub enum TilesetEntry {
    ExternalRef(TilesetRef),
    Inline(Tileset),
}

impl Serialize for TilesetEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TilesetEntry::ExternalRef(r) => r.serialize(serializer),
            TilesetEntry::Inline(ts) => ts.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for TilesetEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Buffer the JSON value to inspect which variant to use
        let value = Value::deserialize(deserializer)?;
        if value.get("$ref").is_some() {
            let r = TilesetRef::deserialize(value).map_err(serde::de::Error::custom)?;
            Ok(TilesetEntry::ExternalRef(r))
        } else {
            let ts = Tileset::deserialize(value).map_err(serde::de::Error::custom)?;
            Ok(TilesetEntry::Inline(ts))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TilesetRef {
    #[serde(rename = "$ref")]
    pub ref_path: String,
    pub first_gid: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tileset {
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub image: String,
    pub image_width: u32,
    pub image_height: u32,
    pub columns: u32,
    pub tile_count: u32,

    #[serde(default)]
    pub margin: u32,

    #[serde(default)]
    pub spacing: u32,

    /// Only present in inline tilesets within a map file.
    /// Absent in standalone .cartile-ts files.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub first_gid: u32,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tiles: HashMap<String, TileData>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileData {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: Properties,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tile: Option<AutoTile>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTile {
    pub group: String,
    pub rule: AutoTileRule,
    pub bitmask: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTileRule {
    #[serde(rename = "bitmask_4bit")]
    Bitmask4bit,
    #[serde(rename = "bitmask_8bit")]
    Bitmask8bit,
}

/// A standalone tileset file (.cartile-ts)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TilesetFile {
    pub cartile: String,

    #[serde(rename = "type")]
    pub file_type: String,

    #[serde(flatten)]
    pub tileset: Tileset,
}
