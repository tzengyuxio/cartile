# cartile CLI — Tiled JSON Converter Design

> Date: 2026-03-22

## Overview

A CLI tool (`cartile`) that converts Tiled map editor's JSON export to the cartile format. First subcommand of the `cartile` CLI binary.

### Scope

**In scope (v0.1):**
- `cartile convert` — Tiled JSON → `.cartile`
- `cartile validate` — validate `.cartile` files
- Tiled embedded tilesets → cartile inline tilesets (default)
- Tiled external tilesets (`.tsj`) → optionally preserved as `$ref` or inlined
- Tile layers (raw int array data)
- Object layers (rect, ellipse, point, polygon, polyline)
- Properties on all levels
- GID bit flag conversion (4-bit Tiled → 3-bit cartile)

**Out of scope:**
- TMX XML format
- LDtk JSON format
- Reverse direction (cartile → Tiled)
- Infinite/chunk-based maps
- Image layers, text objects, template references, Wang sets
- Tile animation, collision shapes

---

## 1. Crate Structure

```
crates/
  cartile-cli/
    Cargo.toml           # binary crate
    src/
      main.rs            # clap entry point + subcommand routing
      commands/
        mod.rs
        convert.rs       # convert subcommand
        validate.rs      # validate subcommand
      tiled/
        mod.rs
        types.rs         # Tiled JSON type definitions (serde structs)
        convert.rs       # Tiled → CartileMap conversion logic
```

**Dependencies:**
- `clap` (derive) — CLI argument parsing
- `anyhow` — error handling (appropriate for binary crates)
- `cartile-format` — core format types and validation
- `serde` + `serde_json` — Tiled JSON deserialization

---

## 2. CLI Interface

```bash
# Convert Tiled JSON to cartile
cartile convert -i map.json -o map.cartile
cartile convert -i map.json                    # default output: map.cartile
cartile convert -i map.json --external-tilesets # keep external tileset refs

# Validate cartile file
cartile validate map.cartile
cartile validate map.cartile --quiet           # exit code only

# Version
cartile --version
```

### `convert` subcommand

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `-i, --input` | yes | — | Input Tiled JSON file path |
| `-o, --output` | no | input with `.cartile` ext | Output cartile file path |
| `--external-tilesets` | no | false | Convert external `.tsj` to `.cartile-ts` and use `$ref` |

Input format detection: check for `tiledversion` field in JSON root. No `--from` flag needed (only Tiled JSON supported).

Default output path: replace input file extension with `.cartile` (e.g., `world.json` → `world.cartile`).

When `--external-tilesets` is set, external tileset files are converted to `.cartile-ts` in the same directory as the **output** `.cartile` file, and the map uses `$ref` with a relative path from the output file to the tileset file.

### `validate` subcommand

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| positional | yes | — | Path to `.cartile` file |
| `--quiet` | no | false | Suppress output, only set exit code |

`validate` runs the full `CartileMap::validate()` check (structural + semantic: GID ranges, height mode consistency, layer data lengths, etc.). It does NOT check referential integrity of `$ref` paths or whether referenced image files exist.

---

## 3. Tiled JSON Types (`tiled/types.rs`)

Serde structs that mirror the Tiled JSON format. Key types:

### TiledMap (root)
```
type: "map"
version: string
tiledversion: string
orientation: "orthogonal" | "isometric" | "staggered" | "hexagonal"
renderorder: string              // ignored in conversion
width, height: u32
tilewidth, tileheight: u32
infinite: bool
staggeraxis: Option<"x" | "y">
staggerindex: Option<"odd" | "even">
hexsidelength: Option<u32>      // ignored in conversion (hex geometry is implicit)
nextlayerid: Option<u32>        // ignored (editor bookkeeping)
nextobjectid: Option<u32>       // ignored (editor bookkeeping)
layers: Vec<TiledLayer>
tilesets: Vec<TiledTilesetEntry>
properties: Option<Vec<TiledProperty>>
```

### TiledTilesetEntry
Two variants (untagged):
- **Embedded:** full tileset definition with `firstgid`, `name`, `image`, etc.
- **External:** only `firstgid` + `source` (path to `.tsj`)

### TiledLayer
Discriminated by `type` field:
- `tilelayer` — `data: Vec<u32>` (raw int array in JSON export), `width`, `height`
- `objectgroup` — `objects: Vec<TiledObject>`
- `group` — `layers: Vec<TiledLayer>` (recursive)
- `imagelayer` — skipped with warning

### TiledObject
Shape determined by presence of fields:
- `point: true` → point
- `ellipse: true` → ellipse
- `polygon: Vec<{x,y}>` → polygon
- `polyline: Vec<{x,y}>` → polyline
- none of above → rect

Fields: `id`, `name`, `type` (class), `x`, `y`, `width`, `height`, `rotation`, `visible`, `properties`, `gid` (for tile objects — skipped with warning in v0.1), `text` (skipped with warning), `template` (skipped with warning).

### TiledProperty
```
name: string
type: "string" | "int" | "float" | "bool" | "color" | "file" | "object" | "class"
value: Value
```

`object` and `class` types are not supported in cartile v0.1 — skip with warning.

---

## 4. Conversion Logic (`tiled/convert.rs`)

### Top-level fields

- `cartile` → hardcoded `"0.1.0"`
- `map_type` → hardcoded `"map"`
- `name` → derived from input filename stem (e.g., `world.json` → `"world"`)

### Map-level conversion

| Tiled `orientation` | cartile `grid.type` | cartile `projection.type` | cartile `stagger` |
|---------------------|---------------------|---------------------------|-------------------|
| `orthogonal` | `square` | `orthogonal` | none |
| `isometric` | `square` | `isometric` | none |
| `staggered` | `square` | `isometric` | from `staggeraxis`/`staggerindex` |
| `hexagonal` | `hexagonal` | `orthogonal` | from `staggeraxis`/`staggerindex` |

**Stagger mapping (applies to both `staggered` and `hexagonal`):**
- `staggeraxis: "x"` → `stagger.axis: "x"`
- `staggeraxis: "y"` → `stagger.axis: "y"`
- `staggerindex: "odd"` → `stagger.index: "odd"`
- `staggerindex: "even"` → `stagger.index: "even"`

Direct 1:1 mapping — Tiled and cartile use the same axis/index semantics.

**Hex orientation (hexagonal maps only):**
- `staggeraxis: "y"` → `orientation: "pointy_top"` (rows are staggered, hex points up)
- `staggeraxis: "x"` → `orientation: "flat_top"` (columns are staggered, hex is sideways)

**Note on `staggered` projection:** Tiled's `staggered` orientation is visually isometric (diamond-shaped tiles in a staggered grid). It maps to cartile `projection.type: "isometric"` with `stagger`, not `"orthogonal"`.

**Always set:**
- `topology` → `"bounded"` (Tiled doesn't support wrapping)
- `height_mode` → `"none"` (Tiled doesn't have elevation concepts)

**Ignored Tiled fields:** `renderorder` (dropped — cartile has no equivalent; warn if non-default `"right-down"`), `hexsidelength` (hex geometry is implicit from tile dimensions), `nextlayerid`/`nextobjectid` (editor bookkeeping).

### GID conversion

Tiled uses 4 flag bits (bits 31-28), cartile uses 3 flag bits (bits 31-29). The GID field in Tiled is 28 bits (0-27), in cartile it is 29 bits (0-28).

Conversion steps:
```
1. Extract Tiled flags:  tiled_flags = raw & 0xF0000000   // bits 31-28
2. Extract Tiled GID:    tiled_gid   = raw & 0x0FFFFFFF   // bits 27-0
3. Check bit 28 (hex rotation): if tiled_flags & 0x10000000 != 0, warn and clear
4. Extract cartile flags: cartile_flags = tiled_flags & 0xE0000000  // bits 31-29 only
5. Combine:              cartile_raw = cartile_flags | tiled_gid
```

This works because:
- Bits 31, 30, 29 (H-flip, V-flip, D-flip) have identical meaning in both formats
- Tiled's bit 28 (hex 120° rotation) has no cartile equivalent — warn and discard
- Tiled GIDs are 28-bit, which fits within cartile's 29-bit GID field

GID 0 in Tiled = empty tile = GID 0 in cartile (same semantics).

### Tileset conversion

**Embedded tilesets** map directly:
- `firstgid` → `first_gid`
- `name`, `image`, `imagewidth`/`imageheight`, `tilewidth`/`tileheight`, `columns`, `tilecount`, `margin`, `spacing` → corresponding cartile fields
- `tiles[].id` → string key in `tiles` map (e.g., tile with `id: 5` → key `"5"`)
- `tiles[].properties` → `tiles["N"].properties`
- `tiles[].animation`, `tiles[].objectgroup`, `wangsets` → skipped with warning

**External tilesets** (`source` field present):
- **Default (inline):** Read the `.tsj` file (resolved relative to the input map file), parse as `TiledTileset`, convert and embed as inline tileset in the cartile map.
- **With `--external-tilesets`:** Convert `.tsj` to `.cartile-ts`, write to the same directory as the output `.cartile` file, use `$ref` with a relative path from the output file.

### Layer conversion

**Tile layers:**
- `data: Vec<u32>` → apply GID conversion to each element → `Vec<TileId>`
- Verify `data.len() == width * height`
- `name` → direct mapping
- `visible`, `opacity` → direct mapping

**Object layers:**
- Each `TiledObject` → `MapObject`
- `id` → direct mapping (preserve Tiled's object IDs; they are already unique within a Tiled map)
- `name` → direct mapping (empty string if absent)
- Shape detection by field presence (see Section 3)
- `x`, `y` in pixels → direct mapping
- `width`, `height` → direct mapping (for rect/ellipse)
- `rotation` → direct mapping (clockwise degrees, pivot at x,y — same convention)
- `polygon`/`polyline` points → `points` array (already relative coordinates)
- `properties` → converted
- Objects with `gid` (tile objects), `text`, or `template` → skip with warning

**Group layers:**
- Recursively flatten: extract all child layers and append to the top-level layer list
- Prepend group name as prefix: child `"trees"` inside group `"decoration"` becomes `"decoration/trees"`
- If flattened name collides with existing layer name, append `_2`, `_3` etc.

**Image layers:**
- Skip entirely, emit warning to stderr

### Property conversion

| Tiled type | cartile type | Notes |
|------------|-------------|-------|
| `string` | `string` | direct |
| `int` | `int` | direct |
| `float` | `float` | direct |
| `bool` | `bool` | direct |
| `color` | `color` | Tiled `#AARRGGBB` → cartile `#RRGGBBAA` (reorder alpha). If Tiled provides `#RRGGBB` (no alpha), append `FF` → `#RRGGBBFF`. |
| `file` | `file` | direct |
| `object` | — | skip with warning |
| `class` | — | skip with warning |

Tiled properties are an array `[{name, type, value}]`, cartile properties are a map `{name: {type, value}}`.

---

## 5. Error Handling

| Situation | Behavior |
|-----------|----------|
| Input file not found / not valid JSON | Error, exit 1 |
| Not Tiled JSON (no `tiledversion` field) | Error, exit 1 |
| `infinite: true` | Error, exit 1 ("infinite/chunk maps not supported") |
| External `.tsj` file not found | Error, exit 1 |
| Post-conversion validation fails | Error, exit 1 (conversion bug) |
| Image layer encountered | Warning to stderr, skip layer |
| Text object / template ref / tile object | Warning to stderr, skip object |
| Wang sets / animation / collision | Warning to stderr, skip data |
| `object` or `class` property type | Warning to stderr, skip property |
| Hex 120° rotation flag (bit 28) | Warning to stderr, clear flag |
| Non-default `renderorder` | Warning to stderr, ignore |
| Output file exists | Overwrite silently |

After successful conversion, automatically run `CartileMap::validate()` on the output. If validation fails, it indicates a conversion bug — report as error.

---

## 6. Testing Strategy

### Unit tests (in `cartile-cli`)
- Tiled JSON type deserialization (various map configurations)
- GID flag conversion (all flag combinations, bit 28 handling)
- Property type/color conversion (including `#RRGGBB` without alpha)
- Shape detection from Tiled object fields
- Group layer flattening (including name collision handling)
- Stagger mapping for both staggered-square and hexagonal maps

### Integration tests
- Full conversion of sample Tiled JSON files → validate output
- External tileset resolution (both inline and `--external-tilesets` modes)
- Warning output for unsupported features
- Error cases (infinite map, missing file, invalid JSON)
- Map name derivation from filename

### Test fixtures
- Minimal orthogonal map
- Map with multiple tilesets
- Map with object layer (various shapes, including id/name preservation)
- Hexagonal map with stagger
- Staggered isometric map
- Map with external tileset reference
