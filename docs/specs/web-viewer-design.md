# Web Viewer Design

> Date: 2026-03-22

## Overview

A browser-based tilemap viewer that can load `.cartile` and Tiled JSON maps, render them on Canvas 2D, and display map metadata in a side panel. Zero dependencies — pure HTML/CSS/JS + cartile-wasm.

### Scope

**In scope:**
- Load `.cartile` files via drag-and-drop or file picker
- Load Tiled `.json` files (auto-converted via WASM)
- Load tileset image files (PNG) alongside map files
- Canvas 2D tile rendering (orthogonal square grid only)
- Tile flip/rotation rendering via canvas transforms
- Pan (drag) and zoom (scroll wheel)
- Layer visibility toggle
- Map info panel (dimensions, tile size, tileset count)
- Tile properties on hover

**Out of scope:**
- Any editing functionality
- Object layer rendering (listed in panel but not drawn)
- Isometric / hexagonal / staggered rendering
- LDtk format support in viewer (CLI only)
- Heightmap visualization
- Auto-tile resolution in viewer

---

## 1. File Structure

```
web/
  index.html          # single page — UI structure
  style.css           # dark theme styling
  app.js              # main app: file loading, rendering, UI
  pkg/                # wasm-pack output (gitignored)
```

Built WASM goes to `web/pkg/` via `wasm-pack build crates/cartile-wasm --target web --out-dir ../../web/pkg`.

No build step for the web app itself — just serve the `web/` directory.

---

## 2. UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  cartile viewer          [Open Files]      Zoom: 100%    │  ← toolbar
├──────────────────────────────────────────┬───────────────┤
│                                          │ Layers        │
│                                          │ ☑ ground      │
│                                          │ ☑ decoration  │
│            Canvas                        │ ☐ collision   │
│         (tile rendering)                 │               │
│                                          │ Map Info      │
│                                          │ 40×30 tiles   │
│                                          │ 16×16 px      │
│                                          │ Tilesets: 2   │
│                                          │               │
│                                          │ Tile (12, 8)  │
│                                          │ terrain: grass│
│                                          │ walkable: true│
├──────────────────────────────────────────┴───────────────┤
│  Cursor: (12, 8)  │  Tile GID: 42  │  Layer: ground     │  ← status bar
└──────────────────────────────────────────────────────────┘
```

### Toolbar
- App name
- "Open Files" button (opens file picker, accepts multiple files)
- Zoom display + zoom in/out buttons

### Canvas area
- Fills remaining space
- Dark background (#0d1117)
- Renders tile layers in order
- Pan via mouse drag, zoom via scroll wheel
- Grid lines toggle (subtle, optional)

### Side panel (right, ~220px)
- **Layers section**: list of all layers with visibility checkbox. Tile layers show ☑/☐ toggle. Object layers listed but marked "(objects)" and not rendered.
- **Map Info section**: map name, dimensions, tile size, tileset count, format version
- **Tile Properties section**: updates on hover. Shows GID, tileset name, local index, and custom properties from tileset definition.

### Status bar
- Cursor tile position (grid coordinates)
- Tile GID under cursor
- Active layer name

---

## 3. File Loading

### Drag-and-drop
User drops files onto the page. Files are classified by extension:
- `.cartile` → parse as CartileMap JSON
- `.json` → attempt Tiled JSON conversion via `convertTiledJson()`
- `.png` / `.jpg` / `.webp` → treat as tileset image

### File picker
"Open Files" button opens a multi-file picker. Same classification logic.

### Tileset image resolution
Map tilesets reference images by relative path (e.g., `"image": "assets/terrain.png"`). The viewer matches loaded image files by filename (basename only, ignoring directory). If an image is missing, tiles from that tileset render as a colored placeholder rectangle with "?" text.

### State after loading
- Map data parsed and stored in JS
- Tileset images loaded as `Image` objects
- Canvas renders immediately
- Side panel populated

---

## 4. Canvas Rendering

### Coordinate system
- Origin (0,0) at top-left of the map
- X increases right, Y increases down
- Each tile occupies `tile_width × tile_height` pixels in map space
- Canvas has a `camera` state: `{ x, y, zoom }` controlling the viewport

### Render loop
Not a continuous animation loop. Re-renders on:
- Initial load
- Pan / zoom change
- Layer visibility toggle

### Per-tile rendering
For each visible tile layer, for each tile:
1. Skip if GID is 0 (empty)
2. Find tileset by GID range
3. Compute local index: `local = gid - first_gid`
4. Compute source rect in tileset image: `sx = (local % columns) * (tile_width + spacing) + margin`, `sy = (local / columns) * (tile_height + spacing) + margin`
5. Compute destination rect: `dx = col * tile_width`, `dy = row * tile_height`
6. Apply flip/rotation if flags present (use canvas `save/translate/scale/restore`)
7. `drawImage(tilesetImg, sx, sy, tw, th, dx, dy, tw, th)`

### Flip/rotation transforms
Extract flags from raw TileId:
- Horizontal flip: `ctx.scale(-1, 1)` + adjust x
- Vertical flip: `ctx.scale(1, -1)` + adjust y
- Diagonal flip: swap x/y axes (90° rotation basis)

Combinations produce 0°/90°/180°/270° rotations per the spec's rotation lookup table.

### Pan and zoom
- **Pan**: mousedown + mousemove on canvas updates `camera.x`, `camera.y`
- **Zoom**: wheel event scales `camera.zoom` (0.25 to 4.0 range), zoomed around cursor position
- Apply camera transform before rendering: `ctx.setTransform(zoom, 0, 0, zoom, -camera.x * zoom, -camera.y * zoom)`

---

## 5. Side Panel Interaction

### Layer visibility
Clicking a layer's checkbox toggles its `visible` flag in the local map data and triggers re-render. Does not modify the file.

### Tile hover
On `mousemove` over canvas:
1. Convert screen coords to map coords using camera transform
2. Compute tile col/row: `col = floor(mapX / tile_width)`, `row = floor(mapY / tile_height)`
3. Look up tile GID in the topmost visible tile layer at that position
4. Resolve tileset and local index
5. Look up tile properties from tileset `tiles[localIndex]`
6. Update side panel "Tile Properties" section and status bar

---

## 6. WASM Integration

The viewer imports cartile-wasm as an ES module:

```javascript
import init, { parseCartileMap, convertTiledJson } from './pkg/cartile_wasm.js';

await init();  // load WASM

// Parse .cartile file
const mapJson = parseCartileMap(cartileFileContent);
const map = JSON.parse(mapJson);

// Convert Tiled JSON
const resultJson = convertTiledJson(tiledFileContent, 'mapname');
const { cartile_json, warnings } = JSON.parse(resultJson);
const map = JSON.parse(cartile_json);
```

---

## 7. Error Handling

| Situation | Behavior |
|-----------|----------|
| No map file dropped | Show welcome screen with instructions |
| Invalid JSON | Show error toast/banner |
| Tiled conversion fails | Show error with message from WASM |
| Missing tileset image | Render placeholder tiles, show warning |
| WASM not loaded | Show "Loading WASM..." then error if fails |

---

## 8. Testing Strategy

No automated tests for the web viewer (it's a visual UI). Verification is manual:

1. Open `web/index.html` via local HTTP server
2. Drop the SRPG fixture (`.cartile` + a dummy tileset PNG) → verify render
3. Drop a Tiled JSON export → verify auto-conversion and render
4. Test pan/zoom
5. Test layer visibility toggle
6. Test tile hover properties
