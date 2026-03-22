use serde::{Deserialize, Serialize};

use super::object::MapObject;
use super::property::Properties;
use crate::tile_id::TileId;

/// A map layer — either tile data, placed objects, or heightmap values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Layer {
    Tile(TileLayer),
    Object(ObjectLayer),
    Heightmap(HeightmapLayer),
}

/// A layer of tile IDs arranged in a flat array (row-major order).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: String,

    #[serde(default = "default_true")]
    pub visible: bool,

    #[serde(default = "default_one")]
    pub opacity: f64,

    #[serde(default)]
    pub elevation: i32,

    #[serde(default = "default_encoding")]
    pub encoding: String,

    pub data: Vec<TileId>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: Properties,
}

/// A layer of placed map objects (entities, triggers, paths, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectLayer {
    pub name: String,

    #[serde(default = "default_true")]
    pub visible: bool,

    #[serde(default = "default_one")]
    pub opacity: f64,

    pub objects: Vec<MapObject>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: Properties,
}

/// A layer of signed integer height values for isometric/3D-ish maps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeightmapLayer {
    pub name: String,

    #[serde(default = "default_true")]
    pub visible: bool,

    #[serde(default = "default_one")]
    pub opacity: f64,

    pub data: Vec<i32>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: Properties,
}

fn default_true() -> bool {
    true
}
fn default_one() -> f64 {
    1.0
}
fn default_encoding() -> String {
    "dense".to_string()
}
