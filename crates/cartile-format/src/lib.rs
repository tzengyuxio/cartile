pub mod autotile;
pub mod error;
pub mod tile_id;
pub mod types;
mod validate;

pub use autotile::{AutoTileIndex, build_autotile_index, resolve_autotiles};
pub use error::{CartileError, ValidationError};
pub use tile_id::TileId;
pub use types::grid::*;
pub use types::layer::*;
pub use types::map::CartileMap;
pub use types::object::*;
pub use types::property::{Properties, Property, PropertyType};
pub use types::tileset::*;

/// Generate the JSON Schema for the CartileMap format as a pretty-printed JSON string.
pub fn generate_map_schema() -> String {
    let schema = schemars::schema_for!(CartileMap);
    serde_json::to_string_pretty(&schema).expect("schema serialization should not fail")
}
