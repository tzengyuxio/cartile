use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct Grid {
    #[serde(rename = "type")]
    pub grid_type: GridType,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<HexOrientation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stagger: Option<Stagger>,

    #[serde(default)]
    pub topology: Topology,

    pub projection: Projection,

    #[serde(default)]
    pub height_mode: HeightMode,

    /// Unknown fields preserved for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridType {
    Square,
    Hexagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HexOrientation {
    FlatTop,
    PointyTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
pub struct Stagger {
    pub axis: StaggerAxis,
    pub index: StaggerIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaggerAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaggerIndex {
    Odd,
    Even,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    #[default]
    Bounded,
    WrapX,
    WrapY,
    WrapXy,
}

#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct Projection {
    #[serde(rename = "type")]
    pub projection_type: ProjectionType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionType {
    Orthogonal,
    Isometric,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeightMode {
    #[default]
    None,
    Stepped,
    Vertex,
}
