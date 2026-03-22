# Cartile Format Core Library — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `cartile-format` Rust crate that can read, write, and validate cartile v0.1 map files.

**Architecture:** A single Rust library crate with serde-based JSON serialization. Types mirror the format spec 1:1. Validation is a separate pass after deserialization — serde handles structure, validation handles semantics (GID ranges, height mode consistency, etc.). Unknown fields are preserved via `serde_json::Value` catch-all.

**Tech Stack:** Rust 1.94+, serde + serde_json, thiserror for errors

**Spec:** `docs/specs/format-spec-v0.1-design.md`

---

## File Structure

```
Cargo.toml                          # workspace root (single crate for now)
crates/
  cartile-format/
    Cargo.toml                      # library crate
    src/
      lib.rs                        # public API re-exports
      types/
        mod.rs                      # module declarations
        map.rs                      # Map top-level struct
        grid.rs                     # Grid, Stagger, Projection, Topology, HeightMode
        tileset.rs                  # Tileset, TilesetRef, TileData, AutoTile
        layer.rs                    # Layer enum, TileLayer, ObjectLayer, HeightmapLayer
        object.rs                   # MapObject, Shape, Point
        property.rs                 # Property, PropertyType
      tile_id.rs                    # TileId: GID encoding/decoding with bit flags
      validate.rs                   # semantic validation (cross-field rules)
      error.rs                      # CartileError, ValidationError
    tests/
      fixtures/
        minimal_map.cartile         # minimal valid map
        srpg_map.cartile            # complete example from spec Section 8
        hex_map.cartile             # hexagonal grid map
        vertex_height.cartile       # vertex heightmap
        external_ref.cartile        # map with external tileset ref
        terrain.cartile-ts          # standalone tileset file
      roundtrip_test.rs             # serialize → deserialize → compare
      validation_test.rs            # validation pass/fail cases
      tile_id_test.rs               # bit flag encoding/decoding
```

**Why a workspace?** Even though there's only one crate now, the workspace layout (`crates/`) makes it painless to add `cartile-cli`, `cartile-godot`, etc. later without restructuring.

---

### Task 1: Project scaffold and error types

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `crates/cartile-format/Cargo.toml`
- Create: `crates/cartile-format/src/lib.rs`
- Create: `crates/cartile-format/src/error.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/*"]
```

- [ ] **Step 2: Create crate Cargo.toml**

```toml
[package]
name = "cartile-format"
version = "0.1.0"
edition = "2024"
description = "Cartile tilemap format — read, write, and validate .cartile files"
license = "MIT"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
pretty_assertions = "1"
```

- [ ] **Step 3: Create error.rs**

```rust
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
    InvalidDataLength { name: String, actual: usize, expected: usize },

    #[error("heightmap layer '{name}': data length {actual} != expected {expected} ((width+1) * (height+1))")]
    InvalidHeightmapLength { name: String, actual: usize, expected: usize },

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

    #[error("auto_tile bitmask {bitmask} invalid for rule '{rule}'")]
    InvalidAutoTileBitmask { bitmask: u8, rule: String },
}
```

- [ ] **Step 4: Create lib.rs with module stubs**

```rust
pub mod error;

pub use error::{CartileError, ValidationError};
```

- [ ] **Step 5: Verify it compiles**

Run: `cd /Users/user/works/cartile && cargo build`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat: scaffold cartile-format crate with error types"
```

---

### Task 2: Core types — Property and TileId

**Files:**
- Create: `crates/cartile-format/src/types/mod.rs`
- Create: `crates/cartile-format/src/types/property.rs`
- Create: `crates/cartile-format/src/tile_id.rs`
- Create: `crates/cartile-format/tests/tile_id_test.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing tests for TileId**

Create `crates/cartile-format/tests/tile_id_test.rs`:

```rust
use cartile_format::tile_id::TileId;

#[test]
fn empty_tile() {
    let tid = TileId::EMPTY;
    assert_eq!(tid.raw(), 0);
    assert!(tid.is_empty());
    assert_eq!(tid.gid(), 0);
    assert!(!tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn simple_gid() {
    let tid = TileId::from_gid(42);
    assert_eq!(tid.gid(), 42);
    assert!(!tid.is_empty());
    assert!(!tid.flip_horizontal());
}

#[test]
fn horizontal_flip() {
    let tid = TileId::from_raw(0x80000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn rotation_90_cw() {
    // 90° CW = horizontal + diagonal
    let tid = TileId::from_raw(0xA0000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(tid.flip_diagonal());
}

#[test]
fn rotation_180() {
    // 180° = horizontal + vertical
    let tid = TileId::from_raw(0xC0000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn max_gid() {
    let tid = TileId::from_gid(0x1FFFFFFF);
    assert_eq!(tid.gid(), 0x1FFFFFFF);
    assert!(!tid.flip_horizontal());
}

#[test]
fn builder_flags() {
    let tid = TileId::new(5, true, false, true);
    assert_eq!(tid.gid(), 5);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(tid.flip_diagonal());
    assert_eq!(tid.raw(), 0xA0000005);
}

#[test]
fn serde_roundtrip() {
    let tid = TileId::from_raw(0x80000001);
    let json = serde_json::to_string(&tid).unwrap();
    assert_eq!(json, "2147483649");
    let back: TileId = serde_json::from_str(&json).unwrap();
    assert_eq!(back.raw(), tid.raw());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tile_id_test`
Expected: FAIL — `tile_id` module not found

- [ ] **Step 3: Implement TileId**

Create `crates/cartile-format/src/tile_id.rs`:

```rust
use serde::{Deserialize, Serialize};

const FLIP_HORIZONTAL: u32 = 0x80000000;
const FLIP_VERTICAL: u32 = 0x40000000;
const FLIP_DIAGONAL: u32 = 0x20000000;
const FLAG_MASK: u32 = 0xE0000000;
const GID_MASK: u32 = 0x1FFFFFFF;

/// A tile identifier that encodes both the global tile ID and transform flags
/// in a single u32. Upper 3 bits are flip flags, lower 29 bits are the GID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TileId(u32);

impl TileId {
    pub const EMPTY: TileId = TileId(0);

    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn from_gid(gid: u32) -> Self {
        debug_assert!(gid <= GID_MASK, "GID exceeds 29-bit maximum");
        Self(gid & GID_MASK)
    }

    pub fn new(gid: u32, flip_h: bool, flip_v: bool, flip_d: bool) -> Self {
        debug_assert!(gid <= GID_MASK, "GID exceeds 29-bit maximum");
        let mut raw = gid & GID_MASK;
        if flip_h {
            raw |= FLIP_HORIZONTAL;
        }
        if flip_v {
            raw |= FLIP_VERTICAL;
        }
        if flip_d {
            raw |= FLIP_DIAGONAL;
        }
        Self(raw)
    }

    pub fn raw(self) -> u32 {
        self.0
    }

    pub fn gid(self) -> u32 {
        self.0 & GID_MASK
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn flip_horizontal(self) -> bool {
        self.0 & FLIP_HORIZONTAL != 0
    }

    pub fn flip_vertical(self) -> bool {
        self.0 & FLIP_VERTICAL != 0
    }

    pub fn flip_diagonal(self) -> bool {
        self.0 & FLIP_DIAGONAL != 0
    }

    pub fn flags(self) -> u32 {
        self.0 & FLAG_MASK
    }
}
```

- [ ] **Step 4: Create property.rs**

Create `crates/cartile-format/src/types/property.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Properties = HashMap<String, Property>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: PropertyType,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Bool,
    String,
    Int,
    Float,
    Color,
    File,
}
```

- [ ] **Step 5: Create types/mod.rs and update lib.rs**

Create `crates/cartile-format/src/types/mod.rs`:

```rust
pub mod property;
```

Update `crates/cartile-format/src/lib.rs`:

```rust
pub mod error;
pub mod tile_id;
pub mod types;

pub use error::{CartileError, ValidationError};
pub use tile_id::TileId;
pub use types::property::{Properties, Property, PropertyType};
```

- [ ] **Step 6: Run tests**

Run: `cargo test --test tile_id_test`
Expected: All 8 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cartile-format/src/tile_id.rs crates/cartile-format/src/types/ crates/cartile-format/src/lib.rs crates/cartile-format/tests/tile_id_test.rs
git commit -m "feat: add TileId (GID + bit flags) and Property types"
```

---

### Task 3: Grid types

**Files:**
- Create: `crates/cartile-format/src/types/grid.rs`
- Modify: `crates/cartile-format/src/types/mod.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing test**

Add to a new file `crates/cartile-format/tests/grid_test.rs`:

```rust
use cartile_format::types::grid::*;

#[test]
fn square_grid_serde_roundtrip() {
    let grid = Grid {
        grid_type: GridType::Square,
        width: 100,
        height: 80,
        tile_width: 16,
        tile_height: 16,
        orientation: None,
        stagger: None,
        topology: Topology::Bounded,
        projection: Projection {
            projection_type: ProjectionType::Orthogonal,
            angle: None,
        },
        height_mode: HeightMode::None,
    };
    let json = serde_json::to_string(&grid).unwrap();
    let back: Grid = serde_json::from_str(&json).unwrap();
    assert_eq!(back.grid_type, GridType::Square);
    assert_eq!(back.width, 100);
    assert_eq!(back.topology, Topology::Bounded);
    assert_eq!(back.height_mode, HeightMode::None);
}

#[test]
fn hex_grid_serde_roundtrip() {
    let grid = Grid {
        grid_type: GridType::Hexagonal,
        width: 20,
        height: 20,
        tile_width: 32,
        tile_height: 28,
        orientation: Some(HexOrientation::PointyTop),
        stagger: Some(Stagger {
            axis: StaggerAxis::Y,
            index: StaggerIndex::Odd,
        }),
        topology: Topology::WrapXy,
        projection: Projection {
            projection_type: ProjectionType::Orthogonal,
            angle: None,
        },
        height_mode: HeightMode::Stepped,
    };
    let json = serde_json::to_string_pretty(&grid).unwrap();
    let back: Grid = serde_json::from_str(&json).unwrap();
    assert_eq!(back.grid_type, GridType::Hexagonal);
    assert_eq!(back.orientation, Some(HexOrientation::PointyTop));
    assert!(back.stagger.is_some());
    assert_eq!(back.topology, Topology::WrapXy);
}

#[test]
fn oblique_projection_with_angle() {
    let json = r#"{"type":"oblique","angle":26.57}"#;
    let proj: Projection = serde_json::from_str(json).unwrap();
    assert_eq!(proj.projection_type, ProjectionType::Oblique);
    assert_eq!(proj.angle, Some(26.57));
}

#[test]
fn topology_default_is_bounded() {
    let json = r#"{
        "type": "square", "width": 10, "height": 10,
        "tile_width": 16, "tile_height": 16,
        "projection": {"type": "orthogonal"}
    }"#;
    let grid: Grid = serde_json::from_str(json).unwrap();
    assert_eq!(grid.topology, Topology::Bounded);
    assert_eq!(grid.height_mode, HeightMode::None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test grid_test`
Expected: FAIL — module not found

- [ ] **Step 3: Implement grid.rs**

Create `crates/cartile-format/src/types/grid.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridType {
    Square,
    Hexagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HexOrientation {
    FlatTop,
    PointyTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stagger {
    pub axis: StaggerAxis,
    pub index: StaggerIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaggerAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaggerIndex {
    Odd,
    Even,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    #[default]
    Bounded,
    WrapX,
    WrapY,
    WrapXy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    #[serde(rename = "type")]
    pub projection_type: ProjectionType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionType {
    Orthogonal,
    Isometric,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeightMode {
    #[default]
    None,
    Stepped,
    Vertex,
}
```

- [ ] **Step 4: Update mod.rs and lib.rs**

Update `crates/cartile-format/src/types/mod.rs`:

```rust
pub mod grid;
pub mod property;
```

Update `crates/cartile-format/src/lib.rs`:

```rust
pub mod error;
pub mod tile_id;
pub mod types;

pub use error::{CartileError, ValidationError};
pub use tile_id::TileId;
pub use types::grid::*;
pub use types::property::{Properties, Property, PropertyType};
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test grid_test`
Expected: All 4 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-format/src/types/grid.rs crates/cartile-format/src/types/mod.rs crates/cartile-format/src/lib.rs crates/cartile-format/tests/grid_test.rs
git commit -m "feat: add Grid, Projection, Topology, HeightMode types"
```

---

### Task 4: Tileset and AutoTile types

**Files:**
- Create: `crates/cartile-format/src/types/tileset.rs`
- Modify: `crates/cartile-format/src/types/mod.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `crates/cartile-format/tests/tileset_test.rs`:

```rust
use cartile_format::types::tileset::*;
use cartile_format::types::property::{Property, PropertyType};

#[test]
fn inline_tileset_serde() {
    let json = r#"{
        "name": "terrain",
        "tile_width": 16,
        "tile_height": 16,
        "image": "assets/terrain.png",
        "image_width": 256,
        "image_height": 256,
        "columns": 16,
        "tile_count": 256,
        "first_gid": 1,
        "tiles": {
            "5": {
                "properties": {
                    "walkable": { "type": "bool", "value": false }
                },
                "auto_tile": {
                    "group": "water",
                    "rule": "bitmask_4bit",
                    "bitmask": 0
                }
            }
        }
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.tile_count, 256);
            assert_eq!(ts.first_gid, 1);
            let tile5 = ts.tiles.get("5").unwrap();
            let at = tile5.auto_tile.as_ref().unwrap();
            assert_eq!(at.group, "water");
            assert_eq!(at.rule, AutoTileRule::Bitmask4bit);
            assert_eq!(at.bitmask, 0);
        }
        _ => panic!("expected inline tileset"),
    }
}

#[test]
fn external_ref_serde() {
    let json = r#"{
        "$ref": "./tilesets/characters.cartile-ts",
        "first_gid": 257
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::ExternalRef(r) => {
            assert_eq!(r.ref_path, "./tilesets/characters.cartile-ts");
            assert_eq!(r.first_gid, 257);
        }
        _ => panic!("expected external ref"),
    }
}

#[test]
fn optional_fields_default() {
    let json = r#"{
        "name": "test",
        "tile_width": 16,
        "tile_height": 16,
        "image": "test.png",
        "image_width": 64,
        "image_height": 64,
        "columns": 4,
        "tile_count": 16,
        "first_gid": 1
    }"#;
    let ts: TilesetEntry = serde_json::from_str(json).unwrap();
    match ts {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.margin, 0);
            assert_eq!(ts.spacing, 0);
            assert!(ts.tiles.is_empty());
        }
        _ => panic!("expected inline"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tileset_test`
Expected: FAIL — module not found

- [ ] **Step 3: Implement tileset.rs**

Create `crates/cartile-format/src/types/tileset.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::property::Properties;

/// A tileset entry in a map file — either inline or an external reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TilesetEntry {
    ExternalRef(TilesetRef),
    Inline(Tileset),
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
#[serde(rename_all = "snake_case")]
pub enum AutoTileRule {
    Bitmask4bit,
    Bitmask8bit,
}

/// A standalone tileset file (.cartile-ts)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TilesetFile {
    pub cartile: String,

    #[serde(rename = "type")]
    pub file_type: String, // always "tileset"

    #[serde(flatten)]
    pub tileset: Tileset,
}
```

- [ ] **Step 4: Update mod.rs and lib.rs**

Update `crates/cartile-format/src/types/mod.rs`:

```rust
pub mod grid;
pub mod property;
pub mod tileset;
```

Add to `crates/cartile-format/src/lib.rs` re-exports:

```rust
pub use types::tileset::*;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test tileset_test`
Expected: All 3 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-format/src/types/tileset.rs crates/cartile-format/src/types/mod.rs crates/cartile-format/src/lib.rs crates/cartile-format/tests/tileset_test.rs
git commit -m "feat: add Tileset, TilesetRef, AutoTile types"
```

---

### Task 5: Layer and Object types

**Files:**
- Create: `crates/cartile-format/src/types/layer.rs`
- Create: `crates/cartile-format/src/types/object.rs`
- Modify: `crates/cartile-format/src/types/mod.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `crates/cartile-format/tests/layer_test.rs`:

```rust
use cartile_format::types::layer::*;
use cartile_format::types::object::*;
use cartile_format::TileId;

#[test]
fn tile_layer_serde() {
    let json = r#"{
        "type": "tile",
        "name": "ground",
        "elevation": 0,
        "encoding": "dense",
        "data": [1, 0, 2, 2147483649]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 4);
            assert!(tl.data[1].is_empty());
            assert_eq!(tl.data[3].gid(), 1);
            assert!(tl.data[3].flip_horizontal());
            assert!(tl.visible); // default
            assert_eq!(tl.opacity, 1.0); // default
        }
        _ => panic!("expected tile layer"),
    }
}

#[test]
fn object_layer_serde() {
    let json = r#"{
        "type": "object",
        "name": "entities",
        "objects": [
            {
                "id": 1,
                "name": "spawn",
                "x": 128.0,
                "y": 64.0,
                "shape": "point"
            },
            {
                "id": 2,
                "name": "zone",
                "x": 0.0,
                "y": 0.0,
                "width": 100.0,
                "height": 50.0,
                "shape": "rect",
                "rotation": 45.0,
                "properties": {
                    "type": { "type": "string", "value": "trigger" }
                }
            },
            {
                "id": 3,
                "name": "path",
                "x": 10.0,
                "y": 20.0,
                "shape": "polyline",
                "points": [{"x": 0, "y": 0}, {"x": 32, "y": 16}]
            }
        ]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Object(ol) => {
            assert_eq!(ol.objects.len(), 3);
            assert_eq!(ol.objects[0].shape, Shape::Point);
            assert_eq!(ol.objects[1].rotation, 45.0);
            assert_eq!(ol.objects[2].shape, Shape::Polyline);
            assert_eq!(ol.objects[2].points.as_ref().unwrap().len(), 2);
        }
        _ => panic!("expected object layer"),
    }
}

#[test]
fn heightmap_layer_serde() {
    let json = r#"{
        "type": "heightmap",
        "name": "terrain_height",
        "data": [0, 1, 2, -1, 0, 3]
    }"#;
    let layer: Layer = serde_json::from_str(json).unwrap();
    match layer {
        Layer::Heightmap(hm) => {
            assert_eq!(hm.name, "terrain_height");
            assert_eq!(hm.data, vec![0, 1, 2, -1, 0, 3]);
        }
        _ => panic!("expected heightmap layer"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test layer_test`
Expected: FAIL

- [ ] **Step 3: Implement object.rs**

Create `crates/cartile-format/src/types/object.rs`:

```rust
use serde::{Deserialize, Serialize};

use super::property::Properties;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    Point,
    Rect,
    Ellipse,
    Polygon,
    Polyline,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

- [ ] **Step 4: Implement layer.rs**

Create `crates/cartile-format/src/types/layer.rs`:

```rust
use serde::{Deserialize, Serialize};

use super::object::MapObject;
use super::property::Properties;
use crate::tile_id::TileId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Layer {
    Tile(TileLayer),
    Object(ObjectLayer),
    Heightmap(HeightmapLayer),
}

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

    /// Unknown fields preserved for forward compatibility.
    /// Note: `#[serde(flatten)]` on inner structs of `#[serde(tag = "type")]` enums
    /// has known serde limitations. If this causes issues, switch to a custom
    /// deserializer or `serde_json::Value` intermediate.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

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

    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

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

    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
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
```

- [ ] **Step 5: Update mod.rs and lib.rs**

Update `crates/cartile-format/src/types/mod.rs`:

```rust
pub mod grid;
pub mod layer;
pub mod object;
pub mod property;
pub mod tileset;
```

Add re-exports to `crates/cartile-format/src/lib.rs`:

```rust
pub use types::layer::*;
pub use types::object::*;
```

- [ ] **Step 6: Run tests**

Run: `cargo test --test layer_test`
Expected: All 3 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cartile-format/src/types/layer.rs crates/cartile-format/src/types/object.rs crates/cartile-format/src/types/mod.rs crates/cartile-format/src/lib.rs crates/cartile-format/tests/layer_test.rs
git commit -m "feat: add Layer, ObjectLayer, HeightmapLayer, MapObject types"
```

---

### Task 6: Top-level Map type and read/write API

**Files:**
- Create: `crates/cartile-format/src/types/map.rs`
- Modify: `crates/cartile-format/src/types/mod.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `crates/cartile-format/tests/roundtrip_test.rs`:

```rust
use cartile_format::*;

#[test]
fn minimal_map_roundtrip() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [
            {
                "type": "tile",
                "name": "ground",
                "data": [0, 0, 0, 0]
            }
        ]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    assert_eq!(map.cartile, "0.1.0");
    assert_eq!(map.name, "test");
    assert_eq!(map.grid.width, 2);

    // round-trip
    let serialized = serde_json::to_string_pretty(&map).unwrap();
    let back: CartileMap = serde_json::from_str(&serialized).unwrap();
    assert_eq!(map, back);
}

#[test]
fn unknown_fields_preserved() {
    let json = r#"{
        "cartile": "0.2.0",
        "type": "map",
        "name": "future",
        "future_field": "hello",
        "grid": {
            "type": "square",
            "width": 1,
            "height": 1,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": []
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let serialized = serde_json::to_string(&map).unwrap();
    assert!(serialized.contains("future_field"));
    assert!(serialized.contains("hello"));
}

#[test]
fn read_from_file() {
    let map = CartileMap::from_file("tests/fixtures/minimal_map.cartile").unwrap();
    assert_eq!(map.name, "minimal");
}

#[test]
fn write_to_file() {
    let map = CartileMap::from_file("tests/fixtures/minimal_map.cartile").unwrap();
    let tmp = std::env::temp_dir().join("cartile_test_output.cartile");
    map.to_file(&tmp).unwrap();
    let back = CartileMap::from_file(&tmp).unwrap();
    assert_eq!(map, back);
    std::fs::remove_file(tmp).ok();
}
```

- [ ] **Step 2: Create test fixture**

Create `crates/cartile-format/tests/fixtures/minimal_map.cartile`:

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "minimal",
  "grid": {
    "type": "square",
    "width": 2,
    "height": 2,
    "tile_width": 16,
    "tile_height": 16,
    "projection": { "type": "orthogonal" }
  },
  "tilesets": [],
  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "data": [0, 0, 0, 0]
    }
  ]
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --test roundtrip_test`
Expected: FAIL — `CartileMap` not found

- [ ] **Step 4: Implement map.rs**

Create `crates/cartile-format/src/types/map.rs`:

```rust
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
    pub map_type: String, // "map"

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
    /// Read a map from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, CartileError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Read a map from a file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, CartileError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Serialize to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, CartileError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Write to a file as pretty-printed JSON.
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), CartileError> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

- [ ] **Step 5: Update mod.rs and lib.rs**

Update `crates/cartile-format/src/types/mod.rs`:

```rust
pub mod grid;
pub mod layer;
pub mod map;
pub mod object;
pub mod property;
pub mod tileset;
```

Update `crates/cartile-format/src/lib.rs`:

```rust
pub mod error;
pub mod tile_id;
pub mod types;

pub use error::{CartileError, ValidationError};
pub use tile_id::TileId;
pub use types::grid::*;
pub use types::layer::*;
pub use types::map::CartileMap;
pub use types::object::*;
pub use types::property::{Properties, Property, PropertyType};
pub use types::tileset::*;
```

- [ ] **Step 6: Run tests**

Run: `cargo test --test roundtrip_test`
Expected: All 4 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cartile-format/src/types/map.rs crates/cartile-format/src/types/mod.rs crates/cartile-format/src/lib.rs crates/cartile-format/tests/
git commit -m "feat: add CartileMap top-level type with read/write API"
```

---

### Task 7: Validation

**Files:**
- Create: `crates/cartile-format/src/validate.rs`
- Create: `crates/cartile-format/tests/validation_test.rs`
- Modify: `crates/cartile-format/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/cartile-format/tests/validation_test.rs`:

```rust
use cartile_format::*;

fn base_map() -> CartileMap {
    serde_json::from_str(r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [{
            "name": "t1",
            "tile_width": 16,
            "tile_height": 16,
            "image": "t.png",
            "image_width": 64,
            "image_height": 64,
            "columns": 4,
            "tile_count": 16,
            "first_gid": 1
        }],
        "layers": [{
            "type": "tile",
            "name": "ground",
            "data": [1, 2, 3, 4]
        }]
    }"#).unwrap()
}

#[test]
fn valid_map_passes() {
    let map = base_map();
    assert!(map.validate().is_ok());
}

#[test]
fn invalid_data_length() {
    let mut map = base_map();
    if let Layer::Tile(ref mut tl) = map.layers[0] {
        tl.data.push(TileId::EMPTY); // now 5 elements, but grid is 2x2=4
    }
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::InvalidDataLength { .. }));
}

#[test]
fn duplicate_layer_name() {
    let mut map = base_map();
    map.layers.push(Layer::Tile(TileLayer {
        name: "ground".to_string(), // duplicate
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data: vec![TileId::EMPTY; 4],
        properties: Default::default(),
    }));
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::DuplicateLayerName { .. }));
}

#[test]
fn hex_missing_orientation() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "hexagonal",
            "width": 2, "height": 2,
            "tile_width": 32, "tile_height": 28,
            "stagger": { "axis": "y", "index": "odd" },
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "data": [0,0,0,0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::MissingHexOrientation));
}

#[test]
fn hex_missing_stagger() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "hexagonal",
            "width": 2, "height": 2,
            "tile_width": 32, "tile_height": 28,
            "orientation": "pointy_top",
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "data": [0,0,0,0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::MissingHexStagger));
}

#[test]
fn stepped_elevation_ok() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2, "height": 2,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" },
            "height_mode": "stepped"
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "elevation": 3, "data": [0,0,0,0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    assert!(map.validate().is_ok());
}

#[test]
fn unexpected_elevation() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2, "height": 2,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "elevation": 3, "data": [0,0,0,0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::UnexpectedElevation { .. }));
}

#[test]
fn vertex_needs_heightmap() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2, "height": 2,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" },
            "height_mode": "vertex"
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "data": [0,0,0,0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::MissingHeightmapLayer));
}

#[test]
fn overlapping_gid_ranges() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 1, "height": 1,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [
            { "name": "a", "tile_width": 16, "tile_height": 16, "image": "a.png",
              "image_width": 64, "image_height": 64, "columns": 4, "tile_count": 16, "first_gid": 1 },
            { "name": "b", "tile_width": 16, "tile_height": 16, "image": "b.png",
              "image_width": 64, "image_height": 64, "columns": 4, "tile_count": 16, "first_gid": 10 }
        ],
        "layers": [{ "type": "tile", "name": "g", "data": [0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::OverlappingGidRanges { .. }));
}

#[test]
fn oblique_missing_angle() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 1, "height": 1,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "oblique" }
        },
        "tilesets": [],
        "layers": [{ "type": "tile", "name": "g", "data": [0] }]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::MissingObliqueAngle));
}

#[test]
fn duplicate_object_id() {
    let json = r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 1, "height": 1,
            "tile_width": 16, "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "tilesets": [],
        "layers": [
            { "type": "object", "name": "a", "objects": [
                { "id": 1, "x": 0, "y": 0, "shape": "point" }
            ]},
            { "type": "object", "name": "b", "objects": [
                { "id": 1, "x": 10, "y": 10, "shape": "point" }
            ]}
        ]
    }"#;
    let map: CartileMap = serde_json::from_str(json).unwrap();
    let err = map.validate().unwrap_err();
    assert!(matches!(err, ValidationError::DuplicateObjectId { .. }));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test validation_test`
Expected: FAIL — `validate` method not found

- [ ] **Step 3: Implement validate.rs**

Create `crates/cartile-format/src/validate.rs`:

```rust
use std::collections::HashSet;

use crate::error::ValidationError;
use crate::types::grid::{GridType, HeightMode, ProjectionType};
use crate::types::layer::Layer;
use crate::types::map::CartileMap;
use crate::types::tileset::TilesetEntry;

impl CartileMap {
    /// Run all semantic validation checks on this map.
    /// Returns the first error found, or Ok(()).
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_grid()?;
        self.validate_tilesets()?;
        self.validate_layers()?;
        self.validate_height_mode()?;
        self.validate_object_ids()?;
        Ok(())
    }

    fn validate_grid(&self) -> Result<(), ValidationError> {
        let grid = &self.grid;

        // Hex requires orientation and stagger
        if grid.grid_type == GridType::Hexagonal {
            if grid.orientation.is_none() {
                return Err(ValidationError::MissingHexOrientation);
            }
            if grid.stagger.is_none() {
                return Err(ValidationError::MissingHexStagger);
            }
        }

        // Square must not have orientation
        if grid.grid_type == GridType::Square && grid.orientation.is_some() {
            return Err(ValidationError::UnexpectedOrientation);
        }

        // Oblique requires angle
        if grid.projection.projection_type == ProjectionType::Oblique {
            match grid.projection.angle {
                None => return Err(ValidationError::MissingObliqueAngle),
                Some(a) if a <= 0.0 || a >= 90.0 => {
                    return Err(ValidationError::ObliqueAngleOutOfRange { angle: a });
                }
                _ => {}
            }
        } else if grid.projection.angle.is_some() {
            return Err(ValidationError::UnexpectedProjectionAngle);
        }

        Ok(())
    }

    fn validate_tilesets(&self) -> Result<(), ValidationError> {
        let mut ranges: Vec<(String, u32, u32)> = Vec::new();

        for entry in &self.tilesets {
            match entry {
                TilesetEntry::Inline(ts) => {
                    if ts.first_gid < 1 {
                        return Err(ValidationError::InvalidFirstGid {
                            name: ts.name.clone(),
                            first_gid: ts.first_gid,
                        });
                    }
                    let end = ts.first_gid.saturating_add(ts.tile_count);
                    if end - 1 > 0x1FFFFFFF {
                        return Err(ValidationError::GidOutOfRange {
                            gid: end - 1,
                        });
                    }
                    ranges.push((ts.name.clone(), ts.first_gid, end));

                    // Validate auto-tile bitmask values
                    self.validate_auto_tiles(ts)?;
                }
                TilesetEntry::ExternalRef(r) => {
                    if r.first_gid < 1 {
                        return Err(ValidationError::InvalidFirstGid {
                            name: r.ref_path.clone(),
                            first_gid: r.first_gid,
                        });
                    }
                }
            }
        }

        // Check for overlapping ranges (inline tilesets only — external refs
        // don't have tile_count so we can't check their full range)
        ranges.sort_by_key(|r| r.1);
        for w in ranges.windows(2) {
            if w[0].2 > w[1].1 {
                return Err(ValidationError::OverlappingGidRanges {
                    a: w[0].0.clone(),
                    b: w[1].0.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_auto_tiles(&self, ts: &crate::types::tileset::Tileset) -> Result<(), ValidationError> {
        use crate::types::tileset::AutoTileRule;

        for tile_data in ts.tiles.values() {
            if let Some(ref at) = tile_data.auto_tile {
                let valid = match at.rule {
                    AutoTileRule::Bitmask4bit => at.bitmask <= 15,
                    AutoTileRule::Bitmask8bit => {
                        // For 8-bit: diagonal only counts if both adjacent
                        // cardinals are present. Validate that the bitmask
                        // satisfies this constraint.
                        let n = at.bitmask & 2 != 0;   // North
                        let w = at.bitmask & 8 != 0;   // West
                        let e = at.bitmask & 16 != 0;  // East
                        let s = at.bitmask & 64 != 0;  // South
                        let nw = at.bitmask & 1 != 0;  // NW
                        let ne = at.bitmask & 4 != 0;  // NE
                        let sw = at.bitmask & 32 != 0;  // SW
                        let se = at.bitmask & 128 != 0; // SE

                        // Each diagonal requires both adjacent cardinals
                        (!nw || (n && w))
                            && (!ne || (n && e))
                            && (!sw || (s && w))
                            && (!se || (s && e))
                    }
                };
                if !valid {
                    return Err(ValidationError::InvalidAutoTileBitmask {
                        bitmask: at.bitmask,
                        rule: format!("{:?}", at.rule),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_layers(&self) -> Result<(), ValidationError> {
        let expected_tile_count = (self.grid.width * self.grid.height) as usize;
        let expected_vertex_count =
            ((self.grid.width + 1) * (self.grid.height + 1)) as usize;

        let mut names = HashSet::new();

        for layer in &self.layers {
            let name = match layer {
                Layer::Tile(tl) => {
                    if tl.data.len() != expected_tile_count {
                        return Err(ValidationError::InvalidDataLength {
                            name: tl.name.clone(),
                            actual: tl.data.len(),
                            expected: expected_tile_count,
                        });
                    }
                    &tl.name
                }
                Layer::Object(ol) => &ol.name,
                Layer::Heightmap(hm) => {
                    if hm.data.len() != expected_vertex_count {
                        return Err(ValidationError::InvalidHeightmapLength {
                            name: hm.name.clone(),
                            actual: hm.data.len(),
                            expected: expected_vertex_count,
                        });
                    }
                    &hm.name
                }
            };

            if !names.insert(name.clone()) {
                return Err(ValidationError::DuplicateLayerName {
                    name: name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_height_mode(&self) -> Result<(), ValidationError> {
        let heightmap_layers: Vec<&str> = self.layers.iter().filter_map(|l| {
            if let Layer::Heightmap(hm) = l {
                Some(hm.name.as_str())
            } else {
                None
            }
        }).collect();

        match self.grid.height_mode {
            HeightMode::Vertex => {
                if heightmap_layers.is_empty() {
                    return Err(ValidationError::MissingHeightmapLayer);
                }
                if heightmap_layers.len() > 1 {
                    return Err(ValidationError::DuplicateLayerName {
                        name: format!("multiple heightmap layers: {:?}", heightmap_layers),
                    });
                }
            }
            HeightMode::None | HeightMode::Stepped => {
                if let Some(&name) = heightmap_layers.first() {
                    return Err(ValidationError::UnexpectedHeightmapLayer {
                        mode: format!("{:?}", self.grid.height_mode).to_lowercase(),
                        name: name.to_string(),
                    });
                }
            }
        }

        // Check elevation usage
        if self.grid.height_mode != HeightMode::Stepped {
            for layer in &self.layers {
                if let Layer::Tile(tl) = layer {
                    if tl.elevation != 0 {
                        return Err(ValidationError::UnexpectedElevation {
                            name: tl.name.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_object_ids(&self) -> Result<(), ValidationError> {
        let mut ids = HashSet::new();
        for layer in &self.layers {
            if let Layer::Object(ol) = layer {
                for obj in &ol.objects {
                    if !ids.insert(obj.id) {
                        return Err(ValidationError::DuplicateObjectId { id: obj.id });
                    }
                }
            }
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Update lib.rs**

Add to `crates/cartile-format/src/lib.rs`:

```rust
mod validate;
```

(Note: `mod validate` not `pub mod` — the public API is the `validate()` method on `CartileMap`, not the module itself.)

- [ ] **Step 5: Run tests**

Run: `cargo test --test validation_test`
Expected: All 11 tests PASS

- [ ] **Step 6: Run all tests**

Run: `cargo test`
Expected: All tests across all test files PASS

- [ ] **Step 7: Commit**

```bash
git add crates/cartile-format/src/validate.rs crates/cartile-format/src/lib.rs crates/cartile-format/tests/validation_test.rs
git commit -m "feat: add semantic validation for CartileMap"
```

---

### Task 8: Full spec example roundtrip and comprehensive fixtures

**Files:**
- Create: `crates/cartile-format/tests/fixtures/srpg_map.cartile`
- Create: `crates/cartile-format/tests/fixtures/hex_map.cartile`
- Create: `crates/cartile-format/tests/fixtures/vertex_height.cartile`
- Create: `crates/cartile-format/tests/fixtures/external_ref.cartile`
- Create: `crates/cartile-format/tests/fixtures/terrain.cartile-ts`
- Modify: `crates/cartile-format/tests/roundtrip_test.rs`

- [ ] **Step 1: Create srpg_map.cartile fixture**

Create `crates/cartile-format/tests/fixtures/srpg_map.cartile` with the complete example from spec Section 8 (the `tutorial_battlefield` JSON). Copy it verbatim from `docs/specs/format-spec-v0.1-design.md` Section 8.

- [ ] **Step 2: Create hex_map.cartile fixture**

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "hex_world",
  "grid": {
    "type": "hexagonal",
    "width": 3,
    "height": 3,
    "tile_width": 32,
    "tile_height": 28,
    "orientation": "pointy_top",
    "stagger": { "axis": "y", "index": "odd" },
    "topology": "wrap_xy",
    "projection": { "type": "orthogonal" }
  },
  "tilesets": [],
  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "data": [0, 0, 0, 0, 0, 0, 0, 0, 0]
    }
  ]
}
```

- [ ] **Step 3: Create vertex_height.cartile fixture**

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "height_test",
  "grid": {
    "type": "square",
    "width": 2,
    "height": 2,
    "tile_width": 32,
    "tile_height": 32,
    "projection": { "type": "oblique", "angle": 26.57 },
    "height_mode": "vertex"
  },
  "tilesets": [],
  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "data": [0, 0, 0, 0]
    },
    {
      "type": "heightmap",
      "name": "terrain_height",
      "data": [0, 0, 0, 1, 2, 1, 0, 1, 0]
    }
  ]
}
```

- [ ] **Step 4: Create external ref fixtures**

`crates/cartile-format/tests/fixtures/terrain.cartile-ts`:

```json
{
  "cartile": "0.1.0",
  "type": "tileset",
  "name": "terrain",
  "tile_width": 16,
  "tile_height": 16,
  "image": "terrain.png",
  "image_width": 64,
  "image_height": 64,
  "columns": 4,
  "tile_count": 16
}
```

`crates/cartile-format/tests/fixtures/external_ref.cartile`:

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "with_ref",
  "grid": {
    "type": "square",
    "width": 1,
    "height": 1,
    "tile_width": 16,
    "tile_height": 16,
    "projection": { "type": "orthogonal" }
  },
  "tilesets": [
    {
      "$ref": "./terrain.cartile-ts",
      "first_gid": 1
    }
  ],
  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "data": [1]
    }
  ]
}
```

- [ ] **Step 5: Add roundtrip tests for all fixtures**

Append to `crates/cartile-format/tests/roundtrip_test.rs`:

```rust
#[test]
fn srpg_map_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/srpg_map.cartile").unwrap();
    assert_eq!(map.name, "tutorial_battlefield");
    assert_eq!(map.grid.height_mode, HeightMode::Stepped);
    assert_eq!(map.tilesets.len(), 1);
    assert_eq!(map.layers.len(), 3);
    assert!(map.validate().is_ok());

    let json = map.to_json_pretty().unwrap();
    let back: CartileMap = serde_json::from_str(&json).unwrap();
    assert_eq!(map, back);
}

#[test]
fn hex_map_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/hex_map.cartile").unwrap();
    assert_eq!(map.grid.grid_type, GridType::Hexagonal);
    assert!(map.validate().is_ok());
}

#[test]
fn vertex_height_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/vertex_height.cartile").unwrap();
    assert_eq!(map.grid.height_mode, HeightMode::Vertex);
    assert!(map.validate().is_ok());
}

#[test]
fn external_ref_roundtrip() {
    let map = CartileMap::from_file("tests/fixtures/external_ref.cartile").unwrap();
    match &map.tilesets[0] {
        TilesetEntry::ExternalRef(r) => {
            assert_eq!(r.ref_path, "./terrain.cartile-ts");
            assert_eq!(r.first_gid, 1);
        }
        _ => panic!("expected external ref"),
    }
}

#[test]
fn tileset_file_roundtrip() {
    use cartile_format::types::tileset::TilesetFile;
    let content = std::fs::read_to_string("tests/fixtures/terrain.cartile-ts").unwrap();
    let tsf: TilesetFile = serde_json::from_str(&content).unwrap();
    assert_eq!(tsf.tileset.name, "terrain");
    assert_eq!(tsf.tileset.tile_count, 16);
}
```

- [ ] **Step 6: Run all tests**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 7: Run clippy and fmt**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: No warnings, no formatting changes

- [ ] **Step 8: Commit**

```bash
git add crates/cartile-format/tests/
git commit -m "test: add comprehensive roundtrip tests and fixture files"
```
