# cartile CLI — Tiled JSON Converter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `cartile` CLI binary with a `convert` command that transforms Tiled JSON exports into `.cartile` map files, and a `validate` command.

**Architecture:** A new `cartile-cli` binary crate in the workspace that depends on `cartile-format`. Tiled JSON types are defined as serde structs. Conversion is a pure function `TiledMap → CartileMap`. CLI uses clap for argument parsing and anyhow for error handling.

**Tech Stack:** Rust, clap (derive), anyhow, serde/serde_json, cartile-format

**Spec:** `docs/specs/cli-tiled-converter-design.md`

---

## File Structure

```
crates/
  cartile-cli/
    Cargo.toml
    src/
      main.rs                 # clap App definition + subcommand dispatch
      commands/
        mod.rs                # re-exports
        convert.rs            # convert subcommand handler
        validate.rs           # validate subcommand handler
      tiled/
        mod.rs                # re-exports
        types.rs              # Tiled JSON serde structs
        convert.rs            # TiledMap → CartileMap pure conversion
    tests/
      fixtures/
        orthogonal.json       # minimal Tiled orthogonal map
        with_objects.json     # map with object layer (various shapes)
        hexagonal.json        # hex map with stagger
        external_tileset.json # map referencing external .tsj
        terrain.tsj           # external tileset file
      convert_test.rs         # integration tests for full conversion pipeline
```

---

### Task 1: CLI crate scaffold + clap setup

**Files:**
- Create: `crates/cartile-cli/Cargo.toml`
- Create: `crates/cartile-cli/src/main.rs`
- Create: `crates/cartile-cli/src/commands/mod.rs`
- Create: `crates/cartile-cli/src/commands/convert.rs`
- Create: `crates/cartile-cli/src/commands/validate.rs`
- Create: `crates/cartile-cli/src/tiled/mod.rs`

- [ ] **Step 1: Create crate Cargo.toml**

```toml
[package]
name = "cartile-cli"
version = "0.1.0"
edition = "2024"
description = "cartile CLI — tilemap format converter and validator"
license = "MIT"

[[bin]]
name = "cartile"
path = "src/main.rs"

[dependencies]
cartile-format = { path = "../cartile-format" }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
pretty_assertions = "1"
```

- [ ] **Step 2: Create main.rs with clap**

```rust
use clap::{Parser, Subcommand};

mod commands;
mod tiled;

#[derive(Parser)]
#[command(name = "cartile", version, about = "Universal tilemap toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a Tiled JSON map to cartile format
    Convert(commands::convert::ConvertArgs),
    /// Validate a cartile map file
    Validate(commands::validate::ValidateArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Validate(args) => commands::validate::run(args),
    }
}
```

- [ ] **Step 3: Create commands/mod.rs**

```rust
pub mod convert;
pub mod validate;
```

- [ ] **Step 4: Create commands/convert.rs (stub)**

```rust
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input Tiled JSON file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output cartile file (default: input with .cartile extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Keep external tilesets as $ref instead of inlining
    #[arg(long)]
    pub external_tilesets: bool,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    anyhow::bail!("convert not yet implemented")
}
```

- [ ] **Step 5: Create commands/validate.rs**

```rust
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to .cartile file to validate
    pub file: PathBuf,

    /// Only set exit code, suppress output
    #[arg(long)]
    pub quiet: bool,
}

pub fn run(args: ValidateArgs) -> anyhow::Result<()> {
    let map = cartile_format::CartileMap::from_file(&args.file)?;
    match map.validate() {
        Ok(()) => {
            if !args.quiet {
                eprintln!("✓ {} is valid", args.file.display());
            }
            Ok(())
        }
        Err(e) => {
            if !args.quiet {
                eprintln!("✗ {}: {e}", args.file.display());
            }
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 6: Create tiled/mod.rs (stub)**

```rust
pub mod types;
pub mod convert;
```

Create empty stub files for `tiled/types.rs` and `tiled/convert.rs` so it compiles:

`tiled/types.rs`:
```rust
// Tiled JSON format types — implemented in Task 2
```

`tiled/convert.rs`:
```rust
// Tiled → CartileMap conversion — implemented in Task 3+
```

- [ ] **Step 7: Verify it builds and runs**

Run: `cargo build -p cartile-cli`
Then: `cargo run -p cartile-cli -- --version`
Expected: prints version

Then: `cargo run -p cartile-cli -- validate crates/cartile-format/tests/fixtures/minimal_map.cartile`
Expected: `✓ ... is valid`

- [ ] **Step 8: Commit**

```bash
git add crates/cartile-cli/
git commit -m "feat: scaffold cartile-cli crate with clap + validate command"
```

---

### Task 2: Tiled JSON type definitions

**Files:**
- Create: `crates/cartile-cli/src/tiled/types.rs`

- [ ] **Step 1: Write failing test**

Create `crates/cartile-cli/tests/fixtures/orthogonal.json`:

```json
{
    "type": "map",
    "version": "1.10",
    "tiledversion": "1.11.2",
    "orientation": "orthogonal",
    "renderorder": "right-down",
    "width": 3,
    "height": 2,
    "tilewidth": 32,
    "tileheight": 32,
    "infinite": false,
    "nextlayerid": 3,
    "nextobjectid": 1,
    "layers": [
        {
            "id": 1,
            "name": "ground",
            "type": "tilelayer",
            "width": 3,
            "height": 2,
            "x": 0,
            "y": 0,
            "visible": true,
            "opacity": 1.0,
            "data": [1, 2, 3, 1, 2, 3]
        }
    ],
    "tilesets": [
        {
            "firstgid": 1,
            "name": "terrain",
            "tilewidth": 32,
            "tileheight": 32,
            "tilecount": 16,
            "columns": 4,
            "image": "terrain.png",
            "imagewidth": 128,
            "imageheight": 128,
            "margin": 0,
            "spacing": 0
        }
    ]
}
```

Create `crates/cartile-cli/tests/tiled_types_test.rs`:

```rust
use std::fs;

#[test]
fn parse_orthogonal_map() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    assert_eq!(map.orientation, "orthogonal");
    assert_eq!(map.width, 3);
    assert_eq!(map.height, 2);
    assert_eq!(map.tilewidth, 32);
    assert!(!map.infinite);
    assert_eq!(map.layers.len(), 1);
    assert_eq!(map.tilesets.len(), 1);
}

#[test]
fn parse_tile_layer() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    match &map.layers[0] {
        cartile_cli::tiled::types::TiledLayer::TileLayer(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0], 1);
            assert!(tl.visible);
        }
        _ => panic!("expected tile layer"),
    }
}

#[test]
fn parse_embedded_tileset() {
    let json = fs::read_to_string("tests/fixtures/orthogonal.json").unwrap();
    let map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(&json).unwrap();
    match &map.tilesets[0] {
        cartile_cli::tiled::types::TiledTilesetEntry::Embedded(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.firstgid, 1);
            assert_eq!(ts.tilecount, 16);
        }
        _ => panic!("expected embedded tileset"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cartile-cli --test tiled_types_test`
Expected: FAIL — types not defined

- [ ] **Step 3: Implement types.rs**

```rust
use serde::Deserialize;

// --- Map ---

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct TiledGroupLayer {
    pub name: String,
    #[serde(default)]
    pub layers: Vec<TiledLayer>,
}

#[derive(Debug, Deserialize)]
pub struct TiledImageLayer {
    pub name: String,
}

// --- Objects ---

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct TiledPoint {
    pub x: f64,
    pub y: f64,
}

// --- Tilesets ---

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TiledTilesetEntry {
    External(TiledExternalTileset),
    Embedded(TiledTileset),
}

#[derive(Debug, Deserialize)]
pub struct TiledExternalTileset {
    pub firstgid: u32,
    pub source: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct TiledTileDef {
    pub id: u32,
    #[serde(default)]
    pub properties: Vec<TiledProperty>,
    pub animation: Option<serde_json::Value>,
    pub objectgroup: Option<serde_json::Value>,
}

// --- Properties ---

#[derive(Debug, Deserialize)]
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
```

- [ ] **Step 4: Update lib exposure for integration tests**

Create `crates/cartile-cli/src/lib.rs`:

```rust
pub mod tiled;
```

This allows integration tests to access `cartile_cli::tiled::types`.

Update `main.rs` to also use the lib module:

```rust
use clap::{Parser, Subcommand};

mod commands;
use cartile_cli::tiled;

// ... rest unchanged
```

Wait — binary crates can't be imported as a library and a binary simultaneously in the same crate without a `lib.rs`. The clean approach: make `main.rs` reference the library.

Actually, the simplest approach: add `lib.rs` that re-exports tiled, and `main.rs` does `use cartile_cli::tiled;` for the conversion logic. But clap commands are in `main.rs`'s module tree. Let's keep it simple:

`src/lib.rs`:
```rust
pub mod tiled;
```

`src/main.rs`:
```rust
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "cartile", version, about = "Universal tilemap toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a Tiled JSON map to cartile format
    Convert(commands::convert::ConvertArgs),
    /// Validate a cartile map file
    Validate(commands::validate::ValidateArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Validate(args) => commands::validate::run(args),
    }
}
```

`commands/convert.rs` uses `cartile_cli::tiled::convert` for the actual logic.

- [ ] **Step 5: Run tests**

Run: `cargo test -p cartile-cli --test tiled_types_test`
Expected: All 3 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-cli/src/tiled/types.rs crates/cartile-cli/src/lib.rs crates/cartile-cli/src/main.rs crates/cartile-cli/tests/
git commit -m "feat: add Tiled JSON type definitions"
```

---

### Task 3: GID conversion + property conversion helpers

**Files:**
- Create: `crates/cartile-cli/src/tiled/convert.rs`
- Modify: `crates/cartile-cli/src/tiled/mod.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/cartile-cli/tests/convert_helpers_test.rs`:

```rust
use cartile_cli::tiled::convert;

#[test]
fn gid_empty() {
    assert_eq!(convert::convert_gid(0), (0, vec![]));
}

#[test]
fn gid_simple() {
    assert_eq!(convert::convert_gid(42), (42, vec![]));
}

#[test]
fn gid_horizontal_flip() {
    // bit 31 set + GID 1
    let raw = 0x80000001u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    assert_eq!(cartile_raw, 0x80000001); // same: bit 31 + GID 1
    assert!(warnings.is_empty());
}

#[test]
fn gid_hex_rotation_flag_cleared() {
    // bit 28 set + GID 5
    let raw = 0x10000005u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    assert_eq!(cartile_raw, 5); // bit 28 cleared
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("hex rotation"));
}

#[test]
fn gid_all_flags() {
    // bits 31,30,29,28 all set + GID 7
    let raw = 0xF0000007u32;
    let (cartile_raw, warnings) = convert::convert_gid(raw);
    // cartile: bits 31,30,29 + GID 7 = 0xE0000007
    assert_eq!(cartile_raw, 0xE0000007);
    assert_eq!(warnings.len(), 1); // hex rotation warning
}

#[test]
fn convert_color_aarrggbb_to_rrggbbaa() {
    assert_eq!(convert::convert_color("#FF336699"), "#336699FF");
}

#[test]
fn convert_color_rrggbb_to_rrggbbff() {
    assert_eq!(convert::convert_color("#336699"), "#336699FF");
}

#[test]
fn convert_property_string() {
    use cartile_format::PropertyType;
    let tp = cartile_cli::tiled::types::TiledProperty {
        name: "key".to_string(),
        property_type: "string".to_string(),
        value: serde_json::json!("hello"),
    };
    let (name, prop, warnings) = convert::convert_property(&tp);
    assert_eq!(name, "key");
    assert_eq!(prop.property_type, PropertyType::String);
    assert!(warnings.is_empty());
}

#[test]
fn convert_property_object_type_skipped() {
    let tp = cartile_cli::tiled::types::TiledProperty {
        name: "target".to_string(),
        property_type: "object".to_string(),
        value: serde_json::json!(42),
    };
    let (_, _, warnings) = convert::convert_property(&tp);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("unsupported"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cartile-cli --test convert_helpers_test`
Expected: FAIL

- [ ] **Step 3: Implement convert.rs helpers**

```rust
use cartile_format::{Properties, Property, PropertyType};

use super::types::TiledProperty;

const TILED_FLAG_MASK: u32 = 0xF000_0000;
const TILED_GID_MASK: u32 = 0x0FFF_FFFF;
const HEX_ROTATION_BIT: u32 = 0x1000_0000;
const CARTILE_FLAG_MASK: u32 = 0xE000_0000;

/// Convert a Tiled GID (with 4 flag bits) to a cartile raw TileId value (3 flag bits).
/// Returns (cartile_raw, warnings).
pub fn convert_gid(tiled_raw: u32) -> (u32, Vec<String>) {
    if tiled_raw == 0 {
        return (0, vec![]);
    }
    let mut warnings = vec![];
    let tiled_flags = tiled_raw & TILED_FLAG_MASK;
    let tiled_gid = tiled_raw & TILED_GID_MASK;

    if tiled_flags & HEX_ROTATION_BIT != 0 {
        warnings.push("tile has hex rotation flag (bit 28) — cleared".to_string());
    }

    let cartile_flags = tiled_flags & CARTILE_FLAG_MASK;
    (cartile_flags | tiled_gid, warnings)
}

/// Convert a Tiled color string to cartile format.
/// Tiled: `#AARRGGBB` or `#RRGGBB`
/// Cartile: `#RRGGBBAA`
pub fn convert_color(tiled_color: &str) -> String {
    let hex = tiled_color.trim_start_matches('#');
    match hex.len() {
        8 => {
            // #AARRGGBB → #RRGGBBAA
            let aa = &hex[0..2];
            let rrggbb = &hex[2..8];
            format!("#{rrggbb}{aa}")
        }
        6 => {
            // #RRGGBB → #RRGGBBFF
            format!("#{hex}FF")
        }
        _ => tiled_color.to_string(), // pass through unknown formats
    }
}

/// Convert a single Tiled property to cartile format.
/// Returns (name, Property, warnings). If the property type is unsupported,
/// a default string property is returned with a warning.
pub fn convert_property(tp: &TiledProperty) -> (String, Property, Vec<String>) {
    let mut warnings = vec![];
    let (prop_type, value) = match tp.property_type.as_str() {
        "string" => (PropertyType::String, tp.value.clone()),
        "int" => (PropertyType::Int, tp.value.clone()),
        "float" => (PropertyType::Float, tp.value.clone()),
        "bool" => (PropertyType::Bool, tp.value.clone()),
        "file" => (PropertyType::File, tp.value.clone()),
        "color" => {
            let converted = tp
                .value
                .as_str()
                .map(|s| serde_json::Value::String(convert_color(s)))
                .unwrap_or_else(|| tp.value.clone());
            (PropertyType::Color, converted)
        }
        other => {
            warnings.push(format!(
                "unsupported property type '{other}' for '{name}' — skipped",
                name = tp.name
            ));
            (PropertyType::String, tp.value.clone())
        }
    };
    let prop = Property {
        property_type: prop_type,
        value,
    };
    (tp.name.clone(), prop, warnings)
}

/// Convert a Vec of Tiled properties to a cartile Properties map.
/// Returns (Properties, warnings). Unsupported types are skipped.
pub fn convert_properties(tiled_props: &[TiledProperty]) -> (Properties, Vec<String>) {
    let mut props = Properties::new();
    let mut all_warnings = vec![];
    for tp in tiled_props {
        let (name, prop, warnings) = convert_property(tp);
        let is_unsupported = !warnings.is_empty();
        all_warnings.extend(warnings);
        if !is_unsupported {
            props.insert(name, prop);
        }
    }
    (props, all_warnings)
}
```

- [ ] **Step 4: Update tiled/mod.rs**

```rust
pub mod convert;
pub mod types;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p cartile-cli --test convert_helpers_test`
Expected: All 9 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-cli/src/tiled/convert.rs crates/cartile-cli/src/tiled/mod.rs crates/cartile-cli/tests/convert_helpers_test.rs
git commit -m "feat: add GID conversion and property conversion helpers"
```

---

### Task 4: TiledMap → CartileMap conversion (core logic)

**Files:**
- Modify: `crates/cartile-cli/src/tiled/convert.rs`

This is the main conversion function. It uses the helpers from Task 3.

- [ ] **Step 1: Write failing test**

Create `crates/cartile-cli/tests/convert_test.rs`:

```rust
use cartile_cli::tiled::convert::convert_tiled_map;
use cartile_cli::tiled::types::TiledMap;
use cartile_format::*;
use std::path::Path;

fn load_tiled(filename: &str) -> TiledMap {
    let path = format!("tests/fixtures/{filename}");
    let json = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn convert_orthogonal_map() {
    let tiled = load_tiled("orthogonal.json");
    let (map, warnings) = convert_tiled_map(&tiled, "orthogonal", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.cartile, "0.1.0");
    assert_eq!(map.map_type, "map");
    assert_eq!(map.name, "orthogonal");
    assert_eq!(map.grid.grid_type, GridType::Square);
    assert_eq!(map.grid.projection.projection_type, ProjectionType::Orthogonal);
    assert_eq!(map.grid.width, 3);
    assert_eq!(map.grid.height, 2);
    assert_eq!(map.grid.tile_width, 32);
    assert_eq!(map.grid.tile_height, 32);
    assert_eq!(map.grid.topology, Topology::Bounded);
    assert_eq!(map.grid.height_mode, HeightMode::None);
    assert!(map.grid.stagger.is_none());
    assert!(map.grid.orientation.is_none());

    // Tileset
    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.first_gid, 1);
            assert_eq!(ts.tile_count, 16);
        }
        _ => panic!("expected inline tileset"),
    }

    // Tile layer
    assert_eq!(map.layers.len(), 1);
    match &map.layers[0] {
        Layer::Tile(tl) => {
            assert_eq!(tl.name, "ground");
            assert_eq!(tl.data.len(), 6);
            assert_eq!(tl.data[0].gid(), 1);
            assert!(tl.visible);
        }
        _ => panic!("expected tile layer"),
    }

    // Should validate
    map.validate().unwrap();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p cartile-cli --test convert_test`
Expected: FAIL — `convert_tiled_map` not found

- [ ] **Step 3: Implement convert_tiled_map**

Add to `crates/cartile-cli/src/tiled/convert.rs` (append after the helpers):

```rust
use std::collections::HashMap;
use std::path::Path;

use cartile_format::{
    CartileMap, Grid, GridType, HeightMode, HexOrientation, Layer, ObjectLayer,
    HeightmapLayer, MapObject, Projection, ProjectionType, Stagger, StaggerAxis,
    StaggerIndex, TileId, TileLayer, Tileset, TilesetEntry, Topology,
};
use cartile_format::{Point, Shape};

use super::types::*;

/// Convert a TiledMap to a CartileMap.
///
/// `map_name` is the name for the output map (typically derived from filename).
/// `input_dir` is the directory containing the input file (for resolving external tilesets).
///
/// Returns (CartileMap, warnings).
/// If `output_dir` is Some, external tilesets are written as `.cartile-ts`
/// files there and referenced via `$ref`. If None, they are inlined.
pub fn convert_tiled_map(
    tiled: &TiledMap,
    map_name: &str,
    input_dir: &Path,
    output_dir: Option<&Path>,
) -> anyhow::Result<(CartileMap, Vec<String>)> {
    let mut warnings = vec![];

    if tiled.infinite {
        anyhow::bail!("infinite/chunk-based maps are not supported");
    }

    // Warn on non-default renderorder
    if let Some(ref ro) = tiled.renderorder {
        if ro != "right-down" {
            warnings.push(format!("renderorder '{ro}' ignored (cartile has no equivalent)"));
        }
    }

    let grid = convert_grid(tiled, &mut warnings)?;
    let tilesets = convert_tilesets(&tiled.tilesets, input_dir, output_dir, &mut warnings)?;
    let layers = convert_layers(&tiled.layers, None, &mut warnings);
    let (properties, prop_warnings) = convert_properties(&tiled.properties);
    warnings.extend(prop_warnings);

    let map = CartileMap {
        cartile: "0.1.0".to_string(),
        map_type: "map".to_string(),
        name: map_name.to_string(),
        properties,
        grid,
        tilesets,
        layers,
        extra: HashMap::new(),
    };

    Ok((map, warnings))
}

fn convert_grid(tiled: &TiledMap, warnings: &mut Vec<String>) -> anyhow::Result<Grid> {
    let (grid_type, projection_type, stagger, orientation) = match tiled.orientation.as_str() {
        "orthogonal" => (GridType::Square, ProjectionType::Orthogonal, None, None),
        "isometric" => (GridType::Square, ProjectionType::Isometric, None, None),
        "staggered" => {
            let stagger = parse_stagger(tiled)?;
            (GridType::Square, ProjectionType::Isometric, Some(stagger), None)
        }
        "hexagonal" => {
            let stagger = parse_stagger(tiled)?;
            let hex_orient = match tiled.staggeraxis.as_deref() {
                Some("x") => HexOrientation::FlatTop,
                Some("y") | _ => HexOrientation::PointyTop,
            };
            (GridType::Hexagonal, ProjectionType::Orthogonal, Some(stagger), Some(hex_orient))
        }
        other => anyhow::bail!("unsupported orientation: {other}"),
    };

    Ok(Grid {
        grid_type,
        width: tiled.width,
        height: tiled.height,
        tile_width: tiled.tilewidth,
        tile_height: tiled.tileheight,
        orientation,
        stagger,
        topology: Topology::Bounded,
        projection: Projection {
            projection_type,
            angle: None,
            extra: HashMap::new(),
        },
        height_mode: HeightMode::None,
        extra: HashMap::new(),
    })
}

fn parse_stagger(tiled: &TiledMap) -> anyhow::Result<Stagger> {
    let axis = match tiled.staggeraxis.as_deref() {
        Some("x") => StaggerAxis::X,
        Some("y") => StaggerAxis::Y,
        other => anyhow::bail!("missing or invalid staggeraxis: {other:?}"),
    };
    let index = match tiled.staggerindex.as_deref() {
        Some("odd") => StaggerIndex::Odd,
        Some("even") => StaggerIndex::Even,
        other => anyhow::bail!("missing or invalid staggerindex: {other:?}"),
    };
    Ok(Stagger { axis, index })
}

fn convert_tilesets(
    tiled_tilesets: &[TiledTilesetEntry],
    input_dir: &Path,
    output_dir: Option<&Path>,
    warnings: &mut Vec<String>,
) -> anyhow::Result<Vec<TilesetEntry>> {
    let mut result = vec![];
    for ts_entry in tiled_tilesets {
        match ts_entry {
            TiledTilesetEntry::Embedded(ts) => {
                result.push(TilesetEntry::Inline(convert_tileset(ts, warnings)));
            }
            TiledTilesetEntry::External(ext) => {
                // Read external .tsj file
                let tsj_path = input_dir.join(&ext.source);
                let json = std::fs::read_to_string(&tsj_path)
                    .map_err(|e| anyhow::anyhow!("failed to read external tileset '{}': {e}", tsj_path.display()))?;
                let mut ts: TiledTileset = serde_json::from_str(&json)?;
                ts.firstgid = ext.firstgid;

                if let Some(out_dir) = output_dir {
                    // --external-tilesets: write .cartile-ts and use $ref
                    let cartile_ts = convert_tileset(&ts, warnings);
                    let ts_filename = Path::new(&ext.source)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("tileset");
                    let ts_out_path = out_dir.join(format!("{ts_filename}.cartile-ts"));

                    let ts_file = cartile_format::types::tileset::TilesetFile {
                        cartile: "0.1.0".to_string(),
                        file_type: "tileset".to_string(),
                        tileset: {
                            let mut t = cartile_ts;
                            t.first_gid = 0; // not present in standalone files
                            t
                        },
                    };
                    let ts_json = serde_json::to_string_pretty(&ts_file)?;
                    std::fs::write(&ts_out_path, ts_json)?;

                    let ref_path = format!("./{ts_filename}.cartile-ts");
                    result.push(TilesetEntry::ExternalRef(
                        cartile_format::TilesetRef {
                            ref_path,
                            first_gid: ext.firstgid,
                        },
                    ));
                } else {
                    // Default: inline the tileset
                    result.push(TilesetEntry::Inline(convert_tileset(&ts, warnings)));
                }
            }
        }
    }
    Ok(result)
}

fn convert_tileset(ts: &TiledTileset, warnings: &mut Vec<String>) -> Tileset {
    let mut tiles = HashMap::new();
    for tile_def in &ts.tiles {
        let mut tile_data_props = HashMap::new();
        let (props, ws) = convert_properties(&tile_def.properties);
        warnings.extend(ws);
        tile_data_props = props;

        if tile_def.animation.is_some() {
            warnings.push(format!("tileset '{}': tile {} animation skipped", ts.name, tile_def.id));
        }
        if tile_def.objectgroup.is_some() {
            warnings.push(format!("tileset '{}': tile {} collision skipped", ts.name, tile_def.id));
        }

        if !tile_data_props.is_empty() {
            tiles.insert(
                tile_def.id.to_string(),
                cartile_format::types::tileset::TileData {
                    properties: tile_data_props,
                    auto_tile: None,
                    extra: HashMap::new(),
                },
            );
        }
    }

    Tileset {
        name: ts.name.clone(),
        tile_width: ts.tilewidth,
        tile_height: ts.tileheight,
        image: ts.image.clone(),
        image_width: ts.imagewidth,
        image_height: ts.imageheight,
        columns: ts.columns,
        tile_count: ts.tilecount,
        margin: ts.margin,
        spacing: ts.spacing,
        first_gid: ts.firstgid,
        tiles,
        extra: HashMap::new(),
    }
}

fn convert_layers(
    tiled_layers: &[TiledLayer],
    group_prefix: Option<&str>,
    warnings: &mut Vec<String>,
) -> Vec<Layer> {
    let mut result = vec![];
    for layer in tiled_layers {
        match layer {
            TiledLayer::TileLayer(tl) => {
                let name = prefixed_name(&tl.name, group_prefix);
                let data: Vec<TileId> = tl.data.iter().map(|&raw| {
                    let (cartile_raw, ws) = convert_gid(raw);
                    warnings.extend(ws);
                    TileId::from_raw(cartile_raw)
                }).collect();

                result.push(Layer::Tile(TileLayer {
                    name,
                    visible: tl.visible,
                    opacity: tl.opacity,
                    elevation: 0,
                    encoding: "dense".to_string(),
                    data,
                    properties: {
                        let (p, ws) = convert_properties(&tl.properties);
                        warnings.extend(ws);
                        p
                    },
                }));
            }
            TiledLayer::ObjectGroup(ol) => {
                let name = prefixed_name(&ol.name, group_prefix);
                let mut objects = vec![];
                for obj in &ol.objects {
                    if obj.gid.is_some() {
                        warnings.push(format!("object '{}' (id={}) is a tile object — skipped", obj.name, obj.id));
                        continue;
                    }
                    if obj.text.is_some() {
                        warnings.push(format!("object '{}' (id={}) is a text object — skipped", obj.name, obj.id));
                        continue;
                    }
                    if obj.template.is_some() {
                        warnings.push(format!("object '{}' (id={}) uses a template — skipped", obj.name, obj.id));
                        continue;
                    }
                    objects.push(convert_object(obj, warnings));
                }

                result.push(Layer::Object(ObjectLayer {
                    name,
                    visible: ol.visible,
                    opacity: ol.opacity,
                    objects,
                    properties: {
                        let (p, ws) = convert_properties(&ol.properties);
                        warnings.extend(ws);
                        p
                    },
                }));
            }
            TiledLayer::Group(gl) => {
                let prefix = match group_prefix {
                    Some(p) => format!("{p}/{}", gl.name),
                    None => gl.name.clone(),
                };
                let children = convert_layers(&gl.layers, Some(&prefix), warnings);
                result.extend(children);
            }
            TiledLayer::ImageLayer(il) => {
                warnings.push(format!("image layer '{}' skipped (not supported)", il.name));
            }
        }
    }
    result
}

fn prefixed_name(name: &str, prefix: Option<&str>) -> String {
    match prefix {
        Some(p) => format!("{p}/{name}"),
        None => name.to_string(),
    }
}

fn deduplicate_layer_name(name: String, existing: &[Layer]) -> String {
    let existing_names: Vec<&str> = existing.iter().map(|l| match l {
        Layer::Tile(tl) => tl.name.as_str(),
        Layer::Object(ol) => ol.name.as_str(),
        Layer::Heightmap(hl) => hl.name.as_str(),
    }).collect();
    if !existing_names.contains(&name.as_str()) {
        return name;
    }
    let mut i = 2;
    loop {
        let candidate = format!("{name}_{i}");
        if !existing_names.contains(&candidate.as_str()) {
            return candidate;
        }
        i += 1;
    }
}

fn convert_object(obj: &TiledObject, warnings: &mut Vec<String>) -> MapObject {
    let (shape, points) = if obj.point {
        (Shape::Point, None)
    } else if obj.ellipse {
        (Shape::Ellipse, None)
    } else if let Some(ref poly) = obj.polygon {
        (Shape::Polygon, Some(poly.iter().map(|p| Point { x: p.x, y: p.y }).collect()))
    } else if let Some(ref poly) = obj.polyline {
        (Shape::Polyline, Some(poly.iter().map(|p| Point { x: p.x, y: p.y }).collect()))
    } else {
        (Shape::Rect, None)
    };

    let (properties, ws) = convert_properties(&obj.properties);
    warnings.extend(ws);

    MapObject {
        id: obj.id,
        name: obj.name.clone(),
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        shape,
        rotation: obj.rotation,
        points,
        properties,
        extra: HashMap::new(),
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p cartile-cli --test convert_test`
Expected: PASS

Run: `cargo test -p cartile-cli`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/cartile-cli/src/tiled/convert.rs crates/cartile-cli/tests/convert_test.rs
git commit -m "feat: implement TiledMap → CartileMap conversion"
```

---

### Task 5: Wire up the convert command + object layer fixture

**Files:**
- Modify: `crates/cartile-cli/src/commands/convert.rs`
- Create: `crates/cartile-cli/tests/fixtures/with_objects.json`
- Modify: `crates/cartile-cli/tests/convert_test.rs`

- [ ] **Step 1: Create object layer fixture**

Create `crates/cartile-cli/tests/fixtures/with_objects.json`:

```json
{
    "type": "map",
    "version": "1.10",
    "tiledversion": "1.11.2",
    "orientation": "orthogonal",
    "renderorder": "right-down",
    "width": 4,
    "height": 4,
    "tilewidth": 16,
    "tileheight": 16,
    "infinite": false,
    "layers": [
        {
            "id": 1,
            "name": "ground",
            "type": "tilelayer",
            "width": 4,
            "height": 4,
            "x": 0,
            "y": 0,
            "visible": true,
            "opacity": 1.0,
            "data": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        },
        {
            "id": 2,
            "name": "objects",
            "type": "objectgroup",
            "visible": true,
            "opacity": 1.0,
            "objects": [
                {
                    "id": 1,
                    "name": "spawn",
                    "x": 32.0,
                    "y": 48.0,
                    "width": 0,
                    "height": 0,
                    "rotation": 0,
                    "visible": true,
                    "point": true
                },
                {
                    "id": 2,
                    "name": "zone",
                    "x": 0.0,
                    "y": 0.0,
                    "width": 64.0,
                    "height": 32.0,
                    "rotation": 45.0,
                    "visible": true
                },
                {
                    "id": 3,
                    "name": "area",
                    "x": 10.0,
                    "y": 20.0,
                    "width": 20.0,
                    "height": 20.0,
                    "rotation": 0,
                    "visible": true,
                    "ellipse": true
                },
                {
                    "id": 4,
                    "name": "path",
                    "x": 0.0,
                    "y": 0.0,
                    "width": 0,
                    "height": 0,
                    "rotation": 0,
                    "visible": true,
                    "polyline": [
                        {"x": 0, "y": 0},
                        {"x": 32, "y": 16},
                        {"x": 64, "y": 0}
                    ]
                },
                {
                    "id": 5,
                    "name": "region",
                    "x": 50.0,
                    "y": 50.0,
                    "width": 0,
                    "height": 0,
                    "rotation": 0,
                    "visible": true,
                    "polygon": [
                        {"x": 0, "y": 0},
                        {"x": 20, "y": 0},
                        {"x": 20, "y": 20},
                        {"x": 0, "y": 20}
                    ]
                }
            ]
        }
    ],
    "tilesets": []
}
```

- [ ] **Step 2: Add tests**

Append to `crates/cartile-cli/tests/convert_test.rs`:

```rust
#[test]
fn convert_object_layer() {
    let tiled = load_tiled("with_objects.json");
    let (map, _) = convert_tiled_map(&tiled, "objects", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.layers.len(), 2);
    match &map.layers[1] {
        Layer::Object(ol) => {
            assert_eq!(ol.objects.len(), 5);
            // Point
            assert_eq!(ol.objects[0].shape, cartile_format::types::object::Shape::Point);
            assert_eq!(ol.objects[0].id, 1);
            assert_eq!(ol.objects[0].name, "spawn");
            // Rect (default)
            assert_eq!(ol.objects[1].shape, cartile_format::types::object::Shape::Rect);
            assert_eq!(ol.objects[1].rotation, 45.0);
            // Ellipse
            assert_eq!(ol.objects[2].shape, cartile_format::types::object::Shape::Ellipse);
            // Polyline
            assert_eq!(ol.objects[3].shape, cartile_format::types::object::Shape::Polyline);
            assert_eq!(ol.objects[3].points.as_ref().unwrap().len(), 3);
            // Polygon
            assert_eq!(ol.objects[4].shape, cartile_format::types::object::Shape::Polygon);
            assert_eq!(ol.objects[4].points.as_ref().unwrap().len(), 4);
        }
        _ => panic!("expected object layer"),
    }
    map.validate().unwrap();
}
```

- [ ] **Step 3: Wire up commands/convert.rs**

```rust
use clap::Args;
use std::path::PathBuf;

use cartile_cli::tiled;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input Tiled JSON file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output cartile file (default: input with .cartile extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Keep external tilesets as $ref instead of inlining
    #[arg(long)]
    pub external_tilesets: bool,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    // Read and parse Tiled JSON
    let json = std::fs::read_to_string(&args.input)?;
    let tiled_map: tiled::types::TiledMap = serde_json::from_str(&json)?;

    // Verify it's a Tiled file
    if tiled_map.tiledversion.is_none() {
        anyhow::bail!("input does not appear to be a Tiled JSON file (missing 'tiledversion')");
    }

    // Derive map name from input filename
    let map_name = args
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let input_dir = args.input.parent().unwrap_or(std::path::Path::new("."));

    // Determine output path early (needed for --external-tilesets)
    let output = args.output.unwrap_or_else(|| args.input.with_extension("cartile"));
    let output_dir = if args.external_tilesets {
        Some(output.parent().unwrap_or(std::path::Path::new(".")))
    } else {
        None
    };

    // Convert
    let (map, warnings) = tiled::convert::convert_tiled_map(&tiled_map, map_name, input_dir, output_dir)?;

    // Print warnings
    for w in &warnings {
        eprintln!("warning: {w}");
    }

    // Validate the output
    if let Err(e) = map.validate() {
        anyhow::bail!("conversion produced invalid output: {e}");
    }

    // Write
    map.to_file(&output)?;
    eprintln!("✓ converted to {}", output.display());

    Ok(())
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p cartile-cli`
Expected: All tests pass

- [ ] **Step 5: Manual test**

Run: `cargo run -p cartile-cli -- convert -i crates/cartile-cli/tests/fixtures/orthogonal.json -o /tmp/test_output.cartile`
Expected: `✓ converted to /tmp/test_output.cartile`

Then: `cargo run -p cartile-cli -- validate /tmp/test_output.cartile`
Expected: `✓ /tmp/test_output.cartile is valid`

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-cli/src/commands/convert.rs crates/cartile-cli/tests/
git commit -m "feat: wire up convert command with object layer support"
```

---

### Task 6: Hex map + external tileset fixtures and tests

**Files:**
- Create: `crates/cartile-cli/tests/fixtures/hexagonal.json`
- Create: `crates/cartile-cli/tests/fixtures/external_tileset.json`
- Create: `crates/cartile-cli/tests/fixtures/terrain.tsj`
- Modify: `crates/cartile-cli/tests/convert_test.rs`

- [ ] **Step 1: Create hex fixture**

`crates/cartile-cli/tests/fixtures/hexagonal.json`:

```json
{
    "type": "map",
    "version": "1.10",
    "tiledversion": "1.11.2",
    "orientation": "hexagonal",
    "renderorder": "right-down",
    "width": 3,
    "height": 3,
    "tilewidth": 32,
    "tileheight": 28,
    "hexsidelength": 14,
    "staggeraxis": "y",
    "staggerindex": "odd",
    "infinite": false,
    "layers": [
        {
            "id": 1,
            "name": "ground",
            "type": "tilelayer",
            "width": 3,
            "height": 3,
            "x": 0,
            "y": 0,
            "visible": true,
            "opacity": 1.0,
            "data": [1, 2, 3, 1, 2, 3, 1, 2, 3]
        }
    ],
    "tilesets": [
        {
            "firstgid": 1,
            "name": "hex_terrain",
            "tilewidth": 32,
            "tileheight": 28,
            "tilecount": 9,
            "columns": 3,
            "image": "hex_terrain.png",
            "imagewidth": 96,
            "imageheight": 84,
            "margin": 0,
            "spacing": 0
        }
    ]
}
```

- [ ] **Step 2: Create external tileset fixtures**

`crates/cartile-cli/tests/fixtures/terrain.tsj`:

```json
{
    "type": "tileset",
    "name": "terrain",
    "tilewidth": 16,
    "tileheight": 16,
    "tilecount": 16,
    "columns": 4,
    "image": "terrain.png",
    "imagewidth": 64,
    "imageheight": 64,
    "margin": 0,
    "spacing": 0
}
```

`crates/cartile-cli/tests/fixtures/external_tileset.json`:

```json
{
    "type": "map",
    "version": "1.10",
    "tiledversion": "1.11.2",
    "orientation": "orthogonal",
    "renderorder": "right-down",
    "width": 2,
    "height": 2,
    "tilewidth": 16,
    "tileheight": 16,
    "infinite": false,
    "layers": [
        {
            "id": 1,
            "name": "ground",
            "type": "tilelayer",
            "width": 2,
            "height": 2,
            "x": 0,
            "y": 0,
            "visible": true,
            "opacity": 1.0,
            "data": [1, 2, 3, 4]
        }
    ],
    "tilesets": [
        {
            "firstgid": 1,
            "source": "terrain.tsj"
        }
    ]
}
```

- [ ] **Step 3: Add tests**

Append to `crates/cartile-cli/tests/convert_test.rs`:

```rust
#[test]
fn convert_hex_map() {
    let tiled = load_tiled("hexagonal.json");
    let (map, _) = convert_tiled_map(&tiled, "hex", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.grid.grid_type, GridType::Hexagonal);
    assert_eq!(map.grid.orientation, Some(HexOrientation::PointyTop));
    assert!(map.grid.stagger.is_some());
    let stagger = map.grid.stagger.unwrap();
    assert_eq!(stagger.axis, StaggerAxis::Y);
    assert_eq!(stagger.index, StaggerIndex::Odd);
    assert_eq!(map.grid.projection.projection_type, ProjectionType::Orthogonal);

    map.validate().unwrap();
}

#[test]
fn convert_external_tileset_inline() {
    let tiled = load_tiled("external_tileset.json");
    let (map, _) = convert_tiled_map(&tiled, "ext", Path::new("tests/fixtures"), None).unwrap();

    assert_eq!(map.tilesets.len(), 1);
    match &map.tilesets[0] {
        TilesetEntry::Inline(ts) => {
            assert_eq!(ts.name, "terrain");
            assert_eq!(ts.first_gid, 1);
            assert_eq!(ts.tile_count, 16);
        }
        _ => panic!("expected inline tileset (external should be resolved)"),
    }
    map.validate().unwrap();
}

#[test]
fn convert_infinite_map_errors() {
    let json = r#"{
        "type": "map", "version": "1.10", "tiledversion": "1.11.2",
        "orientation": "orthogonal", "renderorder": "right-down",
        "width": 10, "height": 10, "tilewidth": 16, "tileheight": 16,
        "infinite": true, "layers": [], "tilesets": []
    }"#;
    let tiled: TiledMap = serde_json::from_str(json).unwrap();
    let result = convert_tiled_map(&tiled, "test", Path::new("."), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("infinite"));
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test -p cartile-cli`
Expected: All tests PASS

- [ ] **Step 5: Run clippy and fmt**

Run: `cargo fmt -p cartile-cli && cargo clippy -p cartile-cli -- -D warnings`
Expected: Clean

- [ ] **Step 6: Commit**

```bash
git add crates/cartile-cli/tests/
git commit -m "test: add hex map, external tileset, and error case tests"
```
