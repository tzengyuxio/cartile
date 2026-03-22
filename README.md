# cartile

*One format. Every engine.*

A universal tilemap toolkit — editor, format, and runtime for every game engine.
Built in Rust with cross-engine bindings via C ABI and WASM.

## What is cartile?

cartile is an open-source tilemap system that provides:

- **A unified tilemap format** — A JSON-based format with schema validation, designed for cross-engine portability and version control friendliness
- **A cross-engine runtime** — A single Rust core library with bindings for Godot, Unity, Bevy, PixiJS, and more. Same behavior, every engine.
- **A CLI toolkit** — Convert between Tiled JSON, LDtk, and cartile format. Validate maps from the command line or CI.
- **A modern editor** — A web-native tilemap editor built with Rust + WASM (coming in Phase 2)

## Why?

| Problem | How cartile solves it |
|---------|----------------------|
| **Tiled's runtimes are community-maintained** — behavior differs across engines | One Rust core, thin binding per engine. 100% consistent. |
| **No standard tilemap format** — TMX is XML-era, LDtk JSON is editor-internal | Purpose-built format with JSON Schema, designed for interop |
| **Editors use legacy tech** — C++/Qt (Tiled) or Haxe/Electron (LDtk) | Rust + WASM: native performance, zero-install web access |
| **Auto-tiling is painful** — 47-tile blob rules are tedious to set up | Bitmask-based auto-tiling with 4-bit and 8-bit modes |

## Quick Start

### CLI Installation

```bash
cargo install --path crates/cartile-cli
```

### Convert a Tiled JSON map

```bash
cartile convert -i mymap.json -o mymap.cartile
```

The CLI auto-detects whether the input is Tiled JSON or LDtk JSON.

### Convert an LDtk project

```bash
cartile convert -i world.ldtk -o world.cartile

# Convert a specific level
cartile convert -i world.ldtk -o level2.cartile --level "Level_2"
```

### Export back to Tiled JSON

```bash
cartile export --to tiled-json -i mymap.cartile -o mymap.json
```

### Validate a map

```bash
cartile validate mymap.cartile
```

Returns exit code 0 on success, 1 on failure. Use `--quiet` to suppress output (useful in CI).

## Rust Library

Add `cartile-format` to your `Cargo.toml`:

```toml
[dependencies]
cartile-format = { path = "crates/cartile-format" }
```

```rust
use cartile_format::CartileMap;

// Load from file
let map = CartileMap::from_file("mymap.cartile")?;

// Or parse from a JSON string
let map = CartileMap::from_json(&json_string)?;

// Validate
map.validate()?;

// Serialize back to JSON
let json = map.to_json_pretty()?;

// Save to file
map.to_file("output.cartile")?;
```

### Auto-tiling

```rust
use cartile_format::{build_autotile_index, resolve_autotiles};

let index = build_autotile_index(&map.tilesets);

for layer in &mut map.layers {
    if let cartile_format::Layer::Tile(tile_layer) = layer {
        resolve_autotiles(tile_layer, &map.grid, &index, &map.tilesets);
    }
}
```

## WASM / JavaScript

Build the WASM package first:

```bash
wasm-pack build crates/cartile-wasm --target web
```

Then use from JavaScript:

```javascript
import init, {
  parseCartileMap,
  validateCartileMap,
  convertTiledJson,
  resolveAutotiles,
} from './cartile_wasm.js';

await init();

// Parse and validate a cartile map
const mapJson = parseCartileMap(cartileJsonString);

// Validate only (throws on error)
validateCartileMap(cartileJsonString);

// Convert a Tiled JSON export to cartile format
const result = convertTiledJson(tiledJsonString, "my_map");
const { cartile_json, warnings } = JSON.parse(result);

// Resolve auto-tiles
const resolved = resolveAutotiles(cartileJsonString);
```

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │         Rust Core Library            │
                    │   (format, auto-tiling, tileset)     │
                    └──────┬──────────────┬────────────────┘
                           │              │
              ┌────────────▼───┐    ┌─────▼──────────┐
              │   C ABI (.so)  │    │  WASM (.wasm)  │
              │   Native path  │    │   Web path     │
              └──┬────┬────┬───┘    └──┬─────────┬───┘
                 │    │    │           │         │
                 ▼    ▼    ▼           ▼         ▼
              Godot Unity Bevy    Web Editor  PixiJS
```

Core logic is written once in Rust. Each engine gets a thin binding layer — no business logic in bindings. This ensures 100% consistent behavior across all engines.

## Format

cartile uses a JSON-based tilemap format (`.cartile` for maps, `.cartile-ts` for standalone tilesets). Key features:

- **Grid types**: square and hexagonal, with stagger support
- **Projections**: orthogonal, isometric, and oblique
- **Layers**: tile layers (dense encoding) and object layers
- **Tile transforms**: flip/rotate via bit flags (same layout as Tiled)
- **Auto-tiling**: bitmask-based, 4-bit (16 tiles) and 8-bit (47 tiles)
- **Custom properties**: typed key-value pairs at map, layer, tile, and object levels
- **Forward-compatible**: unknown fields are preserved on round-trip

See the full [Format Spec v0.1](docs/specs/format-spec-v0.1-design.md) for details.

## Project Structure

```
crates/
├── cartile-format/   # Core library: types, parsing, validation, auto-tiling
├── cartile-cli/      # CLI tool: convert, export, validate
└── cartile-wasm/     # WASM bindings for browser/JS usage
```

## Status

**Phase 1 complete** — Core library, CLI, and WASM bindings are functional.

- Format spec v0.1 (square/hex grids, tile/object/heightmap layers, auto-tiling, custom properties)
- Rust core library with full validation
- CLI: Tiled JSON and LDtk import, Tiled JSON export, validation
- WASM bindings: parse, validate, convert, auto-tile resolve

See [Feasibility Analysis](docs/specs/feasibility-analysis.md) for the full roadmap.

## License

MIT
