use thiserror::Error;

#[derive(Debug, Error)]
pub enum CartileError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("GID {gid} exceeds maximum 0x1FFFFFFF")]
    GidOutOfRange { gid: u32 },

    #[error("tileset '{name}': first_gid must be >= 1, got {first_gid}")]
    InvalidFirstGid { name: String, first_gid: u32 },

    #[error("tileset GID ranges overlap: '{a}' and '{b}'")]
    OverlappingGidRanges { a: String, b: String },

    #[error("layer '{name}': data length {actual} != expected {expected} (width * height)")]
    InvalidDataLength {
        name: String,
        actual: usize,
        expected: usize,
    },

    #[error(
        "heightmap layer '{name}': data length {actual} != expected {expected} ((width+1) * (height+1))"
    )]
    InvalidHeightmapLength {
        name: String,
        actual: usize,
        expected: usize,
    },

    #[error("height_mode is 'vertex' but no heightmap layer found")]
    MissingHeightmapLayer,

    #[error("height_mode is '{mode}' but found heightmap layer '{name}'")]
    UnexpectedHeightmapLayer { mode: String, name: String },

    #[error("height_mode is not 'stepped' but layer '{name}' has non-zero elevation")]
    UnexpectedElevation { name: String },

    #[error("hexagonal grid requires 'orientation' field")]
    MissingHexOrientation,

    #[error("hexagonal grid requires 'stagger' field")]
    MissingHexStagger,

    #[error("square grid must not have 'orientation' field")]
    UnexpectedOrientation,

    #[error("oblique projection requires 'angle' field")]
    MissingObliqueAngle,

    #[error("oblique angle {angle} out of range, must be in (0, 90) exclusive")]
    ObliqueAngleOutOfRange { angle: f64 },

    #[error("non-oblique projection must not have 'angle' field")]
    UnexpectedProjectionAngle,

    #[error("duplicate layer name: '{name}'")]
    DuplicateLayerName { name: String },

    #[error("duplicate object id: {id}")]
    DuplicateObjectId { id: u64 },

    #[error("auto_tile bitmask {bitmask} invalid for rule {rule:?}")]
    InvalidAutoTileBitmask {
        bitmask: u8,
        rule: crate::types::tileset::AutoTileRule,
    },
}
