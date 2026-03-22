use serde::{Deserialize, Serialize};

// --- Map ---

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledMap {
    pub orientation: String,
    pub width: u32,
    pub height: u32,
    pub tilewidth: u32,
    pub tileheight: u32,
    #[serde(default)]
    pub infinite: bool,
    pub tiledversion: Option<String>,
    pub renderorder: Option<String>,
    pub staggeraxis: Option<String>,
    pub staggerindex: Option<String>,
    pub hexsidelength: Option<u32>,
    pub layers: Vec<TiledLayer>,
    pub tilesets: Vec<TiledTilesetEntry>,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
}

// --- Layers ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TiledLayer {
    #[serde(rename = "tilelayer")]
    TileLayer(TiledTileLayer),
    #[serde(rename = "objectgroup")]
    ObjectGroup(TiledObjectLayer),
    #[serde(rename = "group")]
    Group(TiledGroupLayer),
    #[serde(rename = "imagelayer")]
    ImageLayer(TiledImageLayer),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledTileLayer {
    pub name: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_one")]
    pub opacity: f64,
    #[serde(default)]
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledObjectLayer {
    pub name: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_one")]
    pub opacity: f64,
    #[serde(default)]
    pub objects: Vec<TiledObject>,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledGroupLayer {
    pub name: String,
    #[serde(default)]
    pub layers: Vec<TiledLayer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledImageLayer {
    pub name: String,
}

// --- Objects ---

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledObject {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub height: f64,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub point: bool,
    #[serde(default)]
    pub ellipse: bool,
    pub polygon: Option<Vec<TiledPoint>>,
    pub polyline: Option<Vec<TiledPoint>>,
    pub gid: Option<u32>,
    pub text: Option<serde_json::Value>,
    pub template: Option<String>,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TiledPoint {
    pub x: f64,
    pub y: f64,
}

// --- Tilesets ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TiledTilesetEntry {
    External(TiledExternalTileset),
    Embedded(TiledTileset),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledExternalTileset {
    pub firstgid: u32,
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledTileset {
    #[serde(default)]
    pub firstgid: u32,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub tilecount: u32,
    pub columns: u32,
    pub image: String,
    pub imagewidth: u32,
    pub imageheight: u32,
    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub spacing: u32,
    #[serde(default)]
    pub tiles: Vec<TiledTileDef>,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledTileDef {
    pub id: u32,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
    pub animation: Option<serde_json::Value>,
    pub objectgroup: Option<serde_json::Value>,
}

// --- Properties ---

#[derive(Debug, Deserialize, Serialize)]
pub struct TiledProperty {
    pub name: String,
    #[serde(rename = "type", default = "default_string_type")]
    pub property_type: String,
    pub value: serde_json::Value,
}

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

fn default_string_type() -> String {
    "string".to_string()
}
