use serde::{Deserialize, Serialize};

use super::property::Properties;

/// A map object placed in an ObjectLayer, using pixel coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapObject {
    pub id: u64,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    pub x: f64,
    pub y: f64,

    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub width: f64,

    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub height: f64,

    pub shape: Shape,

    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub rotation: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<Point>>,

    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: Properties,

    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// The geometric shape of a map object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    Point,
    Rect,
    Ellipse,
    Polygon,
    Polyline,
}

/// A 2D point used in polygon/polyline definitions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
