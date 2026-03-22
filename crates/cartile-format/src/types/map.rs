use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::grid::Grid;
use super::layer::Layer;
use super::property::Properties;
use super::tileset::TilesetEntry;
use crate::error::CartileError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartileMap {
    pub cartile: String,

    #[serde(rename = "type")]
    pub map_type: String,

    pub name: String,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: Properties,

    pub grid: Grid,

    #[serde(default)]
    pub tilesets: Vec<TilesetEntry>,

    #[serde(default)]
    pub layers: Vec<Layer>,

    /// Catch-all for unknown fields (forward compatibility)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl CartileMap {
    pub fn from_json(json: &str) -> Result<Self, CartileError> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, CartileError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    pub fn to_json_pretty(&self) -> Result<String, CartileError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), CartileError> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
