use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::grid::Grid;
use super::layer::Layer;
use super::property::Properties;
use super::tileset::TilesetEntry;
use crate::error::CartileError;

#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
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
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn to_json_pretty(&self) -> Result<String, CartileError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), CartileError> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
