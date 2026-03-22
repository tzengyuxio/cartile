# cartile Usage Guide

## CLI Reference

The `cartile` CLI provides three commands: `convert`, `export`, and `validate`.

### `cartile convert`

Convert a Tiled JSON or LDtk JSON file to cartile format. The input format is auto-detected.

```
cartile convert -i <INPUT> [-o <OUTPUT>] [--external-tilesets] [--level <NAME>]
```

| Flag | Description |
|------|-------------|
| `-i, --input` | Input file path (Tiled JSON or LDtk JSON) |
| `-o, --output` | Output `.cartile` file (default: input path with `.cartile` extension) |
| `--external-tilesets` | Keep external tilesets as `$ref` instead of inlining (Tiled only) |
| `--level` | LDtk level name to convert (default: first level) |

**Examples:**

```bash
# Convert a Tiled JSON map
cartile convert -i dungeon.json

# Convert with explicit output path
cartile convert -i dungeon.json -o maps/dungeon.cartile

# Keep tilesets as external references
cartile convert -i dungeon.json --external-tilesets

# Convert a specific LDtk level
cartile convert -i world.ldtk --level "Level_2"
```

### `cartile export`

Export a cartile map to another format.

```
cartile export --to <FORMAT> -i <INPUT> [-o <OUTPUT>]
```

| Flag | Description |
|------|-------------|
| `--to` | Target format. Currently supported: `tiled-json` |
| `-i, --input` | Input `.cartile` file |
| `-o, --output` | Output file (default: input path with target format extension) |

**Examples:**

```bash
# Export to Tiled JSON
cartile export --to tiled-json -i mymap.cartile

# Export with explicit output path
cartile export --to tiled-json -i mymap.cartile -o mymap.json
```

### `cartile validate`

Validate a cartile map file against the format spec.

```
cartile validate <FILE> [--quiet]
```

| Flag | Description |
|------|-------------|
| `<FILE>` | Path to `.cartile` file |
| `--quiet` | Suppress output, only set exit code (0 = valid, 1 = invalid) |

**Examples:**

```bash
# Validate a map
cartile validate mymap.cartile

# Validate in CI (quiet mode)
cartile validate mymap.cartile --quiet
```

---

## Supported Input Formats

### Tiled JSON

The CLI accepts Tiled's JSON export format (`.json`). The converter handles:

- Orthogonal, isometric, and hexagonal maps
- Tile layers and object layers
- Multiple tilesets (inline and external TSJ references)
- Tile flip/rotate flags
- Custom properties on maps, layers, tiles, and objects

To export from Tiled as JSON: **File > Export As > JSON map files (*.json)**.

### LDtk JSON

The CLI accepts LDtk project files (`.ldtk`). The converter handles:

- IntGrid layers (converted to tile layers)
- Tile layers and Auto-layers
- Entity layers (converted to object layers)
- Tileset definitions
- Custom fields on entities

Use `--level` to select a specific level. Without it, the first level is converted.

---

## Format Overview

cartile maps are JSON files with the `.cartile` extension. Standalone tilesets use `.cartile-ts`.

### Minimal example

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "my_map",

  "grid": {
    "type": "square",
    "width": 10,
    "height": 10,
    "tile_width": 16,
    "tile_height": 16,
    "topology": "bounded",
    "projection": { "type": "orthogonal" },
    "height_mode": "none"
  },

  "tilesets": [
    {
      "name": "terrain",
      "tile_width": 16,
      "tile_height": 16,
      "image": "terrain.png",
      "image_width": 64,
      "image_height": 64,
      "columns": 4,
      "tile_count": 16,
      "first_gid": 1
    }
  ],

  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "data": [1, 1, 1, 2, 2, 1, 1, 1, 2, 2,
               1, 1, 1, 2, 2, 1, 1, 1, 2, 2,
               3, 3, 1, 1, 1, 1, 1, 3, 3, 3,
               3, 3, 1, 1, 1, 1, 1, 3, 3, 3,
               1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
               1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
               2, 2, 2, 1, 1, 1, 2, 2, 2, 2,
               2, 2, 2, 1, 1, 1, 2, 2, 2, 2,
               1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
               1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    }
  ]
}
```

### Key concepts

- **GID 0** means "no tile" (empty cell)
- **Tile IDs** are global across all tilesets. Each tileset has a `first_gid`; subtract it to get the local tile index.
- **Tile transforms** (flip/rotate) are encoded in the upper 3 bits of tile IDs in layer data. Use bitmask `0x1FFFFFFF` to extract the GID.
- **Layer order** is bottom-to-top in the `layers` array.

### Grid types

| `grid.type` | `grid.stagger` | Result |
|-------------|---------------|--------|
| `"square"` | absent | Regular square grid |
| `"square"` | present | Staggered (checkerboard) grid |
| `"hexagonal"` | required | Hex grid (also requires `orientation`: `"flat_top"` or `"pointy_top"`) |

### Custom properties

Properties use explicit type annotations and are available at map, layer, tile, and object levels:

```json
"properties": {
  "walkable": { "type": "bool", "value": true },
  "terrain": { "type": "string", "value": "grass" },
  "move_cost": { "type": "int", "value": 2 },
  "speed": { "type": "float", "value": 0.8 }
}
```

Supported types: `bool`, `string`, `int`, `float`, `color` (`#RRGGBB` or `#RRGGBBAA`), `file` (relative path).

---

## Auto-tiling

cartile supports bitmask-based auto-tiling in two modes:

### 4-bit (cardinal neighbors)

Checks 4 neighbors (N, E, S, W). Produces 16 tile variants.

```
      N (bit 0)
        ↑
W (bit 3) ← ■ → E (bit 1)
        ↓
      S (bit 2)
```

Bitmask value = sum of bits for connected same-group neighbors. For example, a tile with neighbors to the north and east has bitmask `1 + 2 = 3`.

### 8-bit (all neighbors including diagonals)

Checks 8 neighbors. Diagonals only count if both adjacent cardinals are also connected. This produces the standard 47-tile "blob" tileset.

### Defining auto-tile rules

Auto-tile rules are defined per-tile in the tileset's `tiles` map:

```json
"tiles": {
  "0": {
    "auto_tile": { "group": "grass", "rule": "bitmask_4bit", "bitmask": 0 }
  },
  "1": {
    "auto_tile": { "group": "grass", "rule": "bitmask_4bit", "bitmask": 1 }
  }
}
```

All tiles in the same `group` with the same `rule` form a complete auto-tile set. At runtime, the engine examines neighbors and selects the tile whose `bitmask` matches the neighbor configuration.

### Resolving auto-tiles in code

**Rust:**

```rust
use cartile_format::{build_autotile_index, resolve_autotiles, Layer};

let index = build_autotile_index(&map.tilesets);

for layer in &mut map.layers {
    if let Layer::Tile(tile_layer) = layer {
        resolve_autotiles(tile_layer, &map.grid, &index, &map.tilesets);
    }
}
```

**JavaScript (WASM):**

```javascript
const resolved = resolveAutotiles(mapJsonString);
```

---

## Library API Examples

### Loading and saving

```rust
use cartile_format::CartileMap;

// From file
let map = CartileMap::from_file("level1.cartile")?;

// From JSON string
let map = CartileMap::from_json(r#"{"cartile":"0.1.0", ...}"#)?;

// Validate
map.validate()?;

// Save
map.to_file("output.cartile")?;

// Serialize to JSON string
let json = map.to_json_pretty()?;
```

### Accessing map data

```rust
use cartile_format::{CartileMap, Layer, TileId};

let map = CartileMap::from_file("level1.cartile")?;

// Grid info
println!("Map size: {}x{}", map.grid.width, map.grid.height);

// Iterate layers
for layer in &map.layers {
    match layer {
        Layer::Tile(tile_layer) => {
            println!("Tile layer: {}", tile_layer.name);
            // Access tile data
            for &raw in &tile_layer.data {
                let tile_id = TileId::from_raw(raw);
                if !tile_id.is_empty() {
                    let gid = tile_id.gid();
                    let flipped_h = tile_id.flip_horizontal();
                    // ...
                }
            }
        }
        Layer::Object(obj_layer) => {
            println!("Object layer: {}", obj_layer.name);
            for obj in &obj_layer.objects {
                println!("  {} at ({}, {})", obj.name, obj.x, obj.y);
            }
        }
        Layer::Heightmap(hm_layer) => {
            println!("Heightmap layer: {}", hm_layer.name);
        }
    }
}
```

### Reading custom properties

```rust
use cartile_format::PropertyType;

if let Some(prop) = map.properties.get("max_turns") {
    if prop.property_type == PropertyType::Int {
        println!("Max turns: {}", prop.value);
    }
}
```

---

## Further Reading

- [Format Spec v0.1](specs/format-spec-v0.1-design.md) — Full format specification
- [Feasibility Analysis](specs/feasibility-analysis.md) — Market analysis and product roadmap
- [Tilemap Tools Landscape](research/tilemap-tools-landscape.md) — Competitive landscape research
