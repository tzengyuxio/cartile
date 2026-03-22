# Cartile Format Spec v0.1 — Design Document

> Date: 2026-03-22

## Overview

This document defines the cartile map format v0.1 — a JSON-based, engine-agnostic tilemap format designed for cross-engine portability, version control friendliness, and human readability.

### Scope

**v0.1 includes:**
- Grid types: square, hexagonal
- Grid variations: staggered (checkerboard, hex stagger)
- Map topology: bounded, wrap-x, wrap-y, wrap-xy (toroidal)
- Projection: orthogonal, isometric, oblique
- Height systems: none, stepped (Tactics Ogre style), vertex heightmap (SimCity 2000 style)
- Tile layers (dense encoding) and object layers
- Tileset definitions: inline or external reference
- Tile transforms: flip/rotate via bit flags
- Bitmask-based auto-tiling (4-bit and 8-bit)
- Custom properties at map, layer, tile, and object levels

**Reserved for future versions (not implemented in v0.1):**
- Animated tiles
- Multi-tile objects
- Sparse tile encoding
- World / multi-map composition
- Chunk-based streaming

---

## 1. File Structure

### File extensions

| Extension | Content |
|-----------|---------|
| `.cartile` | Map file (type: "map") |
| `.cartile-ts` | Standalone tileset file (type: "tileset") |

### Top-level schema

```jsonc
{
  "cartile": "0.1.0",           // format version (semver)
  "type": "map",                // "map" | "tileset"
  "name": "overworld",
  "properties": { ... },        // map-level custom properties (Section 7)

  "grid": { ... },              // grid definition (Section 2)
  "tilesets": [ ... ],          // tileset definitions or references (Section 3)
  "layers": [ ... ]             // layer stack (Section 4)
}
```

### Canonical key ordering

To maximize version-control friendliness, writers SHOULD emit keys in the order shown in this spec (e.g., `cartile` → `type` → `name` → `properties` → `grid` → `tilesets` → `layers`). Readers MUST NOT depend on key order.

### Forward compatibility

- Unknown fields MUST be preserved on round-trip. Parsers MUST NOT reject unknown keys.
- When reading a file with a higher minor version (e.g., 0.2.0 read by a 0.1.0 parser), unrecognized fields are preserved and unrecognized enum values trigger a warning, not an error.

---

## 2. Grid Definition

```jsonc
"grid": {
  "type": "square",              // "square" | "hexagonal"
  "width": 100,                  // map width in tiles
  "height": 80,                  // map height in tiles
  "tile_width": 16,              // tile width in pixels
  "tile_height": 16,             // tile height in pixels

  "orientation": "pointy_top",   // hex only: "flat_top" | "pointy_top" (required for hex)

  "stagger": {                   // optional for square, required for hexagonal
    "axis": "y",                 // "x" | "y" — stagger direction
    "index": "odd"               // "odd" | "even" — which row/col is offset
  },

  "topology": "bounded",         // "bounded" | "wrap_x" | "wrap_y" | "wrap_xy"

  "projection": {                // visual projection
    "type": "orthogonal"         // "orthogonal" | "isometric" | "oblique"
    // "angle" is required when type is "oblique", forbidden otherwise
  },

  "height_mode": "none"          // "none" | "stepped" | "vertex"
}
```

### Grid types and stagger

| `type` | `stagger` | `orientation` | Result |
|--------|-----------|---------------|--------|
| `"square"` | absent | N/A | Regular square grid |
| `"square"` | present | N/A | Checkerboard / staggered square grid |
| `"hexagonal"` | required | `"flat_top"` | Flat-top hex grid with stagger |
| `"hexagonal"` | required | `"pointy_top"` | Pointy-top hex grid with stagger |

- `orientation` is **required** when `type` is `"hexagonal"`, **forbidden** when `type` is `"square"`.
- `stagger` is **required** when `type` is `"hexagonal"` (hex grids inherently need stagger configuration), **optional** when `type` is `"square"`.
- All enum values in this format use `snake_case` (e.g., `wrap_x`, `flat_top`, `pointy_top`).

### Projection

| `projection.type` | `projection.angle` | Description |
|--------------------|---------------------|-------------|
| `"orthogonal"` | forbidden | Top-down or side view |
| `"isometric"` | forbidden | Standard isometric (implicit 30° from horizontal) |
| `"oblique"` | required (float, exclusive range (0, 90) degrees) | Angled view (e.g., SimCity 2000 style). Angle is the camera tilt from horizontal toward top-down. 0° would be pure side view (invalid), 90° would be pure top-down (use orthogonal instead). Typical values: 26.57° (SimCity 2000), 30° (common oblique). |

### Topology

| Value | Description |
|-------|-------------|
| `"bounded"` | Map has hard edges (default) |
| `"wrap_x"` | Left edge connects to right edge |
| `"wrap_y"` | Top edge connects to bottom edge |
| `"wrap_xy"` | Toroidal: wraps in both axes |

### Height modes

| `height_mode` | Description | Data location |
|---------------|-------------|---------------|
| `"none"` | Pure 2D, no height concept (default) | — |
| `"stepped"` | Tactics Ogre style. Discrete elevation levels per tile layer | `elevation` field on tile layers |
| `"vertex"` | SimCity 2000 style. Per-vertex heights creating continuous slopes | Dedicated `heightmap` layer type |

These are fundamentally different height systems with different data structures. The mode is declared at the grid level so parsers and runtimes can select the appropriate processing logic.

**Design decisions:**
- Checkerboard is not a separate grid type — it is `square` + `stagger`. This avoids combinatorial explosion of grid types.
- `topology` is a grid-level property, not per-layer. The entire map shares one topology.
- `projection` is orthogonal to grid shape — the same square grid can be rendered orthogonal or isometric.

---

## 3. Tileset Definition and Reference

### Inline tileset

```jsonc
{
  "name": "terrain",
  "tile_width": 16,
  "tile_height": 16,
  "image": "assets/terrain.png",   // relative path from the containing file
  "image_width": 256,
  "image_height": 256,
  "columns": 16,                    // tiles per row in the image
  "tile_count": 256,                // total number of tiles in this tileset
  "margin": 0,                      // outer margin in pixels
  "spacing": 0,                     // spacing between tiles in pixels
  "first_gid": 1,                   // global tile ID start for this tileset
  "tiles": {                        // per-tile data (optional, sparse map)
    "5": {
      "properties": {
        "walkable": { "type": "bool", "value": false },
        "terrain": { "type": "string", "value": "water" }
      },
      "auto_tile": {
        "group": "water",
        "rule": "bitmask_4bit"
      }
    }
  }
}
```

### External reference

```jsonc
{
  "$ref": "./tilesets/characters.cartile-ts",
  "first_gid": 257
}
```

### Standalone tileset file (`.cartile-ts`)

```jsonc
{
  "cartile": "0.1.0",
  "type": "tileset",
  "name": "terrain",
  "tile_width": 16,
  "tile_height": 16,
  "image": "terrain.png",
  "image_width": 256,
  "image_height": 256,
  "columns": 16,
  "tile_count": 256,
  "margin": 0,
  "spacing": 0,
  "tiles": { ... }
}
```

### Required and optional fields

**Inline tileset required fields:** `name`, `tile_width`, `tile_height`, `image`, `image_width`, `image_height`, `columns`, `tile_count`, `first_gid`

**Inline tileset optional fields (with defaults):** `margin` (default: 0), `spacing` (default: 0), `tiles` (default: empty)

**Standalone tileset required fields:** Same as inline, minus `first_gid`, plus `cartile` and `type`.

**External reference required fields:** `$ref`, `first_gid`

### GID allocation

- **GID 0 is reserved and means "no tile".** It MUST NOT be used as a `first_gid` value. The minimum valid `first_gid` is 1.
- **Valid GID range: 1 to 536,870,911 (0x1FFFFFFF).** The upper 3 bits of a 32-bit value are reserved for tile transform flags (Section 5), so GIDs are limited to 29 bits. `first_gid + tile_count - 1` MUST NOT exceed 0x1FFFFFFF.
- Each tileset occupies the GID range `[first_gid, first_gid + tile_count)`.
- GID ranges of different tilesets MUST NOT overlap.
- Layer `data` values of `0` are equivalent to empty (`-1` is NOT used as the empty sentinel; `0` is the empty sentinel — see Section 4).

**Design decisions:**
- **`first_gid` (global tile ID start):** When multiple tilesets coexist, each occupies an ID range. Tile IDs in layer data are global; subtract `first_gid` to get the local index within the tileset. This is a proven mechanism from Tiled.
- **`tiles` uses string-keyed map, not array:** Only tiles with extra data need an entry. Keys are local tile index as string.
- **`image` is a relative path:** Relative to the file that contains the path (map file for inline tilesets, `.cartile-ts` file for standalone tilesets).
- **`$ref` for external reference:** References a standalone `.cartile-ts` file. `first_gid` is specified on the map side because the same tileset may have different GID ranges in different maps.
- **`tile_count`:** Allows parsers to validate GID ranges without loading the image file.

---

## 4. Layer Stack

The `layers` array defines rendering order (bottom → top). Tile layers and object layers can be freely interleaved.

### Common layer fields

All layer types share these fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | string | yes | — | `"tile"`, `"object"`, or `"heightmap"` |
| `name` | string | yes | — | Layer name. MUST be unique within the map. |
| `visible` | bool | no | `true` | Whether the layer is visible |
| `opacity` | float | no | `1.0` | Layer opacity (0.0 to 1.0) |
| `properties` | object | no | `{}` | Layer-level custom properties |

### Tile layer

```jsonc
{
  "type": "tile",
  "name": "ground",
  "visible": true,
  "opacity": 1.0,
  "elevation": 0,                   // integer, only meaningful in "stepped" height_mode
  "encoding": "dense",              // "dense" (v0.1 only), defaults to "dense" if omitted
  "data": [1, 1, 1, 2, 3, 0, 0, 1, ...],
  "properties": { ... }
}
```

- `data` length MUST equal `grid.width × grid.height`
- `0` represents an empty tile (no tile at this position)
- Non-zero values are unsigned 32-bit integers: the lower 29 bits are the global tile ID, the upper 3 bits are transform flags (see Section 5)
- In JSON, these values are written as regular numbers. Parsers MUST treat them as unsigned 32-bit integers. Values range from `0` to `4294967295` (0xFFFFFFFF). All values within this range are valid JSON numbers (well within the IEEE 754 safe integer range of 2^53 - 1).
- `elevation` is only meaningful when `grid.height_mode` is `"stepped"`. It defaults to `0` if omitted.
- `encoding` defaults to `"dense"` if omitted. v0.1 only supports `"dense"`.

### Object layer

```jsonc
{
  "type": "object",
  "name": "entities",
  "objects": [
    {
      "id": 1,
      "name": "player_spawn",
      "x": 128.0,                   // pixel coordinates (float)
      "y": 64.0,
      "width": 16.0,                // optional, for sized shapes
      "height": 16.0,
      "shape": "rect",              // "rect" | "ellipse" | "point" | "polygon" | "polyline"
      "rotation": 0.0,              // degrees, default 0.0
      "points": [ ... ],            // only for "polygon" and "polyline" shapes
      "properties": { ... }
    }
  ]
}
```

- Objects use pixel coordinates (float), not grid coordinates — they are not bound to the grid.
- Object `id` MUST be unique **across the entire map** (not just within a single layer). This enables global lookup by ID.
- `name` is not required to be unique (multiple objects may share the same name).

#### Object fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | int | yes | — | Unique object ID (globally unique within the map) |
| `name` | string | no | `""` | Human-readable name |
| `x` | float | yes | — | X position in pixels |
| `y` | float | yes | — | Y position in pixels |
| `shape` | string | yes | — | Shape type (see table below) |
| `width` | float | no | `0.0` | Width in pixels (for rect/ellipse) |
| `height` | float | no | `0.0` | Height in pixels (for rect/ellipse) |
| `rotation` | float | no | `0.0` | Rotation in degrees (clockwise), pivot is the object's `(x, y)` position |
| `points` | array | conditional | — | Required for polygon/polyline shapes |
| `properties` | object | no | `{}` | Object-level custom properties |

#### Shape types

| Shape | Required fields | Description |
|-------|-----------------|-------------|
| `"point"` | `x`, `y` | A single point. `width`/`height` ignored. |
| `"rect"` | `x`, `y`, `width`, `height` | Axis-aligned rectangle (before rotation). |
| `"ellipse"` | `x`, `y`, `width`, `height` | Ellipse bounded by the rectangle. |
| `"polygon"` | `x`, `y`, `points` | Closed polygon. Last point implicitly connects back to first. Minimum 3 points. |
| `"polyline"` | `x`, `y`, `points` | Open polyline (not closed). Minimum 2 points. |

- `points` contains `[{"x": 0, "y": 0}, {"x": 32, "y": 16}, ...]` — pixel offsets relative to the object's `x, y` position.

### Heightmap layer

Only valid when `grid.height_mode` is `"vertex"`. At most one heightmap layer per map.

```jsonc
{
  "type": "heightmap",
  "name": "terrain_height",
  "data": [0, 0, 1, 1, 2, 2, 1, 0, ...]
}
```

**Heightmap-specific required fields:** `data`

**Heightmap-specific behavior:** `visible` and `opacity` from the common layer fields apply (controlling debug visualization in editors). `properties` is available for metadata.

- `data` length MUST equal `(grid.width + 1) × (grid.height + 1)` (vertex count, one more than tile count in each dimension)
- Values are signed 32-bit integers (i32). Negative values are valid (e.g., below sea level). Range: -2,147,483,648 to 2,147,483,647.

### Height mode validation

- If `grid.height_mode` is `"vertex"`, the map MUST contain exactly one `"heightmap"` layer.
- If `grid.height_mode` is `"none"` or `"stepped"`, the map MUST NOT contain a `"heightmap"` layer.
- If `grid.height_mode` is `"stepped"`, tile layers MAY have `elevation` fields. If `grid.height_mode` is not `"stepped"`, `elevation` MUST be absent or `0`.

---

## 5. Tile Transforms

Tile transforms are encoded as bit flags in the upper bits of tile IDs within layer data:

| Bit | Mask | Meaning |
|-----|------|---------|
| 31 | `0x80000000` | Horizontal flip |
| 30 | `0x40000000` | Vertical flip |
| 29 | `0x20000000` | Diagonal flip (basis for 90° rotation) |
| 0-28 | `0x1FFFFFFF` | Global tile ID |

To extract tile ID and flags from a raw value:

```
flags = raw_value & 0xE0000000
tile_gid = raw_value & 0x1FFFFFFF
```

A raw value of `0` always means empty (no tile). Since GID 0 is reserved, there is no ambiguity between "empty" and "tile 0 with no transforms."

**Rotation lookup:**

| Transform | Horizontal | Vertical | Diagonal |
|-----------|-----------|----------|----------|
| No transform | 0 | 0 | 0 |
| 90° clockwise | 1 | 0 | 1 |
| 180° | 1 | 1 | 0 |
| 270° clockwise (= 90° CCW) | 0 | 1 | 1 |
| Horizontal flip | 1 | 0 | 0 |
| Vertical flip | 0 | 1 | 0 |

**Design decisions:**
- Bit flags instead of per-tile objects: keeps `data` as a flat integer array. Using `{"id": 1, "flip_h": true}` per tile would inflate file size 5-10x.
- Same bit layout as Tiled: reduces TMX conversion complexity and leverages developer familiarity.
- GID 0 reserved as empty sentinel eliminates the signed/unsigned ambiguity that would arise from using `-1`.

---

## 6. Auto-tiling

Auto-tiling rules are defined per-tile within tilesets:

```jsonc
"tiles": {
  "0": {
    "auto_tile": {
      "group": "grass",
      "rule": "bitmask_4bit",
      "bitmask": 0
    }
  },
  "1": {
    "auto_tile": {
      "group": "grass",
      "rule": "bitmask_4bit",
      "bitmask": 1
    }
  }
}
```

### How it works

1. Tiles in the same `group` with the same `rule` form a complete auto-tile set.
2. Each tile in the set declares its `bitmask` value — the neighbor configuration it represents.
3. When a tile is placed, the runtime examines its neighbors that share the same group.
4. The neighbor configuration produces a bitmask value, which maps to the tile with the matching `bitmask`.

### Bitmask bit assignment

**`bitmask_4bit`** — 4 cardinal neighbors, 16 combinations (tiles 0-15):

```
      bit 0 (1)
        ↑
bit 3 (8) ← ■ → bit 1 (2)
        ↓
      bit 2 (4)
```

| Bit | Value | Direction |
|-----|-------|-----------|
| 0 | 1 | North (up) |
| 1 | 2 | East (right) |
| 2 | 4 | South (down) |
| 3 | 8 | West (left) |

A bitmask value of `0` = isolated tile (no same-group neighbors). A value of `15` (0b1111) = surrounded on all four sides.

**`bitmask_8bit`** — 8 neighbors including diagonals:

```
bit 0 (1)   bit 1 (2)   bit 2 (4)
    ↖           ↑           ↗
bit 3 (8)   ← ■ →   bit 4 (16)
    ↙           ↓           ↘
bit 5 (32)  bit 6 (64)  bit 7 (128)
```

| Bit | Value | Direction |
|-----|-------|-----------|
| 0 | 1 | North-West |
| 1 | 2 | North |
| 2 | 4 | North-East |
| 3 | 8 | West |
| 4 | 16 | East |
| 5 | 32 | South-West |
| 6 | 64 | South |
| 7 | 128 | South-East |

For 8-bit mode, a diagonal neighbor is only considered "connected" if both adjacent cardinal neighbors are also connected. For example, NW (bit 0) only counts if both N (bit 1) and W (bit 3) are present. This reduces the theoretical 256 combinations to **47 unique tiles** (the standard "blob" tileset convention).

A tileset using `bitmask_4bit` MUST map each `bitmask` value to exactly one tile. Valid `bitmask` values for `bitmask_4bit` are unsigned integers 0–15. Valid `bitmask` values for `bitmask_8bit` are the 47 values that satisfy the diagonal rule above (unsigned integers, subset of 0–255). Parsers MUST reject `bitmask` values outside the valid set for the given rule.

A tileset SHOULD provide tiles for all valid bitmask values of its rule. If a bitmask value has no matching tile at runtime, the runtime MUST use the isolated tile (bitmask 0) as fallback.

**Design decisions:**
- Auto-tile rules belong to the tileset, not the map. Rules travel with the tileset — no need to reconfigure when using it in a different map.
- `group` is a string label, not an index. Multiple tile IDs belong to the same group, forming the complete set of variants.
- Each tile explicitly declares its `bitmask` value rather than relying on tile index ordering. This is more robust and allows non-contiguous tile layouts in the tileset image.

---

## 7. Custom Properties

Custom properties use a unified structure across all four levels: map, layer, tile (in tileset), and object.

```jsonc
"properties": {
  "walkable":         { "type": "bool",   "value": true },
  "terrain":          { "type": "string", "value": "water" },
  "move_cost":        { "type": "int",    "value": 3 },
  "speed_multiplier": { "type": "float",  "value": 0.5 },
  "color":            { "type": "color",  "value": "#FF3366AA" },
  "script":           { "type": "file",   "value": "./scripts/on_enter.lua" }
}
```

### Supported types

| Type | JSON value type | Description |
|------|----------------|-------------|
| `bool` | boolean | `true` / `false` |
| `string` | string | Arbitrary text |
| `int` | number | Integer value |
| `float` | number | Floating-point value |
| `color` | string | `#RRGGBB` or `#RRGGBBAA` (alpha is optional, defaults to `FF`) |
| `file` | string | Relative path from the containing file |

### Property inheritance (tileset → map)

Tileset per-tile properties serve as defaults. In v0.1, there is no per-position property override mechanism in tile layers — tile properties are read-only from the tileset definition. Per-position overrides may be added in a future version.

To achieve position-specific data in v0.1, use an object layer with objects placed at the tile positions and custom properties on those objects.

**Design decisions:**
- Explicit type annotation: each property has `type` + `value`, rather than relying on JSON type inference. This eliminates `3` vs `3.0` ambiguity and enables precise JSON Schema validation.
- Identical structure at all four levels: callers use a single API to read/write properties regardless of where they are attached.
- `file` paths are relative to the file that contains them (map file or tileset file), consistent with `image` path resolution.

---

## 8. Complete Example

A small 4×3 SRPG map demonstrating key features:

```json
{
  "cartile": "0.1.0",
  "type": "map",
  "name": "tutorial_battlefield",
  "properties": {
    "author": { "type": "string", "value": "cartile-team" },
    "max_turns": { "type": "int", "value": 20 }
  },

  "grid": {
    "type": "square",
    "width": 4,
    "height": 3,
    "tile_width": 32,
    "tile_height": 32,
    "topology": "bounded",
    "projection": { "type": "isometric" },
    "height_mode": "stepped"
  },

  "tilesets": [
    {
      "name": "terrain",
      "tile_width": 32,
      "tile_height": 32,
      "image": "assets/terrain.png",
      "image_width": 128,
      "image_height": 128,
      "columns": 4,
      "tile_count": 16,
      "margin": 0,
      "spacing": 0,
      "first_gid": 1,
      "tiles": {
        "0": {
          "properties": {
            "terrain": { "type": "string", "value": "grass" },
            "move_cost": { "type": "int", "value": 1 }
          },
          "auto_tile": { "group": "grass", "rule": "bitmask_4bit", "bitmask": 0 }
        },
        "1": {
          "properties": {
            "terrain": { "type": "string", "value": "stone" },
            "move_cost": { "type": "int", "value": 2 }
          }
        },
        "2": {
          "properties": {
            "terrain": { "type": "string", "value": "water" },
            "walkable": { "type": "bool", "value": false }
          }
        }
      }
    }
  ],

  "layers": [
    {
      "type": "tile",
      "name": "ground",
      "elevation": 0,
      "encoding": "dense",
      "data": [1, 1, 2, 2, 1, 1, 1, 3, 1, 2, 2, 1]
    },
    {
      "type": "tile",
      "name": "high_ground",
      "elevation": 2,
      "encoding": "dense",
      "data": [0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0]
    },
    {
      "type": "object",
      "name": "units",
      "objects": [
        {
          "id": 1,
          "name": "player_spawn",
          "x": 16.0,
          "y": 16.0,
          "shape": "point",
          "properties": {
            "type": { "type": "string", "value": "spawn" },
            "team": { "type": "string", "value": "player" }
          }
        },
        {
          "id": 2,
          "name": "enemy_spawn",
          "x": 112.0,
          "y": 48.0,
          "shape": "point",
          "properties": {
            "type": { "type": "string", "value": "spawn" },
            "team": { "type": "string", "value": "enemy" }
          }
        }
      ]
    }
  ]
}
```
