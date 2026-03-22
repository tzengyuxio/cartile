use serde::Deserialize;

/// Root LDtk project file.
#[derive(Debug, Deserialize)]
pub struct LdtkRoot {
    /// Header info injected by LDtk (contains `app: "LDtk"`)
    #[serde(rename = "__header__")]
    pub header: Option<LdtkHeader>,

    /// JSON format version
    #[serde(rename = "jsonVersion")]
    pub json_version: Option<String>,

    /// Default grid size for the project
    #[serde(rename = "defaultGridSize", default = "default_grid_size")]
    pub default_grid_size: u32,

    /// All tileset definitions
    pub defs: LdtkDefs,

    /// All levels (single-world projects have levels directly here)
    #[serde(default)]
    pub levels: Vec<LdtkLevel>,

    /// Multi-world projects store levels inside worlds
    #[serde(default)]
    pub worlds: Vec<LdtkWorld>,
}

#[derive(Debug, Deserialize)]
pub struct LdtkHeader {
    pub app: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LdtkDefs {
    #[serde(default)]
    pub tilesets: Vec<LdtkTilesetDef>,
}

#[derive(Debug, Deserialize)]
pub struct LdtkTilesetDef {
    pub uid: u32,
    pub identifier: String,
    #[serde(rename = "pxWid")]
    pub px_wid: u32,
    #[serde(rename = "pxHei")]
    pub px_hei: u32,
    #[serde(rename = "tileGridSize")]
    pub tile_grid_size: u32,
    #[serde(rename = "relPath")]
    pub rel_path: Option<String>,
    #[serde(default)]
    pub padding: u32,
    #[serde(default)]
    pub spacing: u32,
}

/// An LDtk world (used in multi-world mode).
#[derive(Debug, Deserialize)]
pub struct LdtkWorld {
    pub identifier: String,
    #[serde(default)]
    pub levels: Vec<LdtkLevel>,
}

/// A single level in an LDtk project.
#[derive(Debug, Deserialize)]
pub struct LdtkLevel {
    pub identifier: String,
    #[serde(rename = "pxWid")]
    pub px_wid: u32,
    #[serde(rename = "pxHei")]
    pub px_hei: u32,
    pub uid: u32,

    /// Layer instances — only present if the project was saved with
    /// "Separate level files" disabled (the common case).
    #[serde(rename = "layerInstances")]
    pub layer_instances: Option<Vec<LdtkLayerInstance>>,
}

/// A layer instance within a level.
#[derive(Debug, Deserialize)]
pub struct LdtkLayerInstance {
    /// Layer type: "Tiles", "IntGrid", "Entities", "AutoLayer"
    #[serde(rename = "__type")]
    pub layer_type: String,

    /// Human-readable layer name
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Grid-based width of this layer
    #[serde(rename = "__cWid")]
    pub c_wid: u32,

    /// Grid-based height of this layer
    #[serde(rename = "__cHei")]
    pub c_hei: u32,

    /// Grid cell size in pixels
    #[serde(rename = "__gridSize")]
    pub grid_size: u32,

    /// UID of the tileset used by this layer (if any)
    #[serde(rename = "__tilesetDefUid")]
    pub tileset_def_uid: Option<u32>,

    /// Tile data for "Tiles" layers
    #[serde(rename = "gridTiles", default)]
    pub grid_tiles: Vec<LdtkGridTile>,

    /// Auto-layer tiles (for "AutoLayer" and "IntGrid" layers with auto-tiling)
    #[serde(rename = "autoLayerTiles", default)]
    pub auto_layer_tiles: Vec<LdtkGridTile>,

    /// IntGrid CSV data — flat array of int values (row-major)
    #[serde(rename = "intGridCsv", default)]
    pub int_grid_csv: Vec<i64>,

    /// Entity instances for "Entities" layers
    #[serde(rename = "entityInstances", default)]
    pub entity_instances: Vec<LdtkEntityInstance>,
}

/// A single tile placement in LDtk.
#[derive(Debug, Deserialize)]
pub struct LdtkGridTile {
    /// Pixel coordinates of the tile in the layer `[x, y]`
    pub px: [i64; 2],

    /// Tile ID in the tileset
    #[serde(rename = "t")]
    pub tile_id: u32,

    /// Flip bits: 0=none, 1=flip-x, 2=flip-y, 3=both
    #[serde(rename = "f", default)]
    pub flip: u32,
}

/// An entity instance in an Entities layer.
#[derive(Debug, Deserialize)]
pub struct LdtkEntityInstance {
    /// Entity definition identifier (name)
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Pixel position `[x, y]`
    pub px: [f64; 2],

    /// Entity width in pixels
    #[serde(default)]
    pub width: f64,

    /// Entity height in pixels
    #[serde(default)]
    pub height: f64,

    /// Unique instance ID
    #[serde(rename = "iid")]
    pub iid: Option<String>,
}

fn default_grid_size() -> u32 {
    16
}
