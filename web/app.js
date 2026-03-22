// ============================================================
// cartile editor — Main application
// ============================================================

let wasmReady = false;
let wasmModule = null;

// ============================================================
// State
// ============================================================
let mapData = null;           // parsed CartileMap object
let tilesetImages = {};       // basename -> HTMLImageElement
let camera = { x: 0, y: 0, zoom: 1.0 };
let layerVisibility = {};     // layer name -> bool
let hoveredTile = null;       // { col, row, gid, tilesetName, localIndex, properties }

// Pan state
let isPanning = false;
let panStart = { x: 0, y: 0 };

// Canvas and context
let canvas, ctx;

// Paint mode state
let currentMode = 'view';       // 'view' | 'paint'
let selectedTile = null;        // { gid, tilesetIndex, localIndex }
let activeTilesetIndex = 0;     // which tileset is shown in panel
let tilesetCanvas, tilesetCtx;  // tileset panel canvas
let isDirty = false;            // true if map has been modified

// Active layer (for painting target)
let selectedLayerIndex = 0;

// Grid overlay
let showGrid = false;

// Undo/Redo
let undoStack = [];  // { layerIndex, idx, oldValue }
let redoStack = [];
const MAX_UNDO = 100;

// Help overlay
let helpVisible = false;

// Pending tileset file for new map modal
let pendingTilesetFile = null;

// Flip/rotation bitmask constants
const FLIP_H = 0x80000000;
const FLIP_V = 0x40000000;
const FLIP_D = 0x20000000;
const GID_MASK = 0x1FFFFFFF;

// Zoom limits
const ZOOM_MIN = 0.25;
const ZOOM_MAX = 4.0;
const ZOOM_STEP = 0.1;

// ============================================================
// Initialization
// ============================================================
async function main() {
    canvas = document.getElementById('map-canvas');
    ctx = canvas.getContext('2d');

    tilesetCanvas = document.getElementById('tileset-canvas');
    tilesetCtx = tilesetCanvas.getContext('2d');

    resizeCanvas();
    setupEventListeners();

    try {
        const module = await import('./pkg/cartile_wasm.js');
        await module.default();
        wasmModule = module;
        wasmReady = true;
    } catch (e) {
        console.warn('WASM not available:', e);
        showToast(
            'WASM module not loaded. Run: wasm-pack build crates/cartile-wasm --target web --out-dir ../../web/pkg'
        );
    }
}

// ============================================================
// Mode Switching
// ============================================================
function setMode(mode) {
    currentMode = mode;
    document.getElementById('btn-mode-view').classList.toggle('active', mode === 'view');
    document.getElementById('btn-mode-paint').classList.toggle('active', mode === 'paint');
    canvas.classList.toggle('paint-mode', mode === 'paint');

    // Show/hide tileset panel
    const tilesetPanel = document.getElementById('tileset-panel');
    if (mode === 'paint' && mapData) {
        tilesetPanel.classList.remove('hidden');
        renderTilesetPanel();
    } else {
        tilesetPanel.classList.add('hidden');
    }
    resizeCanvas();
}

// ============================================================
// Help Overlay
// ============================================================
function toggleHelp() {
    helpVisible = !helpVisible;
    document.getElementById('help-overlay').classList.toggle('hidden', !helpVisible);
}

function hideHelp() {
    helpVisible = false;
    document.getElementById('help-overlay').classList.add('hidden');
}

// ============================================================
// Tileset Panel
// ============================================================
function renderTilesetPanel() {
    if (!mapData || !mapData.tilesets || mapData.tilesets.length === 0) return;

    // Populate tileset select dropdown
    const select = document.getElementById('tileset-select');
    select.innerHTML = '';
    for (let i = 0; i < mapData.tilesets.length; i++) {
        const ts = mapData.tilesets[i];
        const opt = document.createElement('option');
        opt.value = i;
        opt.textContent = ts.name || 'Tileset ' + i;
        select.appendChild(opt);
    }
    select.value = activeTilesetIndex;

    renderTilesetCanvas();
}

function renderTilesetCanvas() {
    const ts = mapData.tilesets[activeTilesetIndex];
    if (!ts) return;

    const imagePath = ts.image || '';
    const basename = imagePath.split('/').pop().toLowerCase();
    const img = tilesetImages[basename];

    if (!img) {
        // Show "no image" message
        tilesetCanvas.width = 200;
        tilesetCanvas.height = 40;
        tilesetCtx.fillStyle = '#7d8590';
        tilesetCtx.font = '12px sans-serif';
        tilesetCtx.fillText('Tileset image not loaded', 10, 25);
        return;
    }

    // Draw tileset image at scaled size for visibility
    const scale = Math.max(1, Math.floor(64 / (ts.tile_width || 16)));
    tilesetCanvas.width = img.width * scale;
    tilesetCanvas.height = img.height * scale;
    tilesetCtx.imageSmoothingEnabled = false;
    tilesetCtx.drawImage(img, 0, 0, tilesetCanvas.width, tilesetCanvas.height);

    // Draw grid lines
    const tw = (ts.tile_width || 16) * scale;
    const th = (ts.tile_height || 16) * scale;
    const spacing = (ts.spacing || 0) * scale;
    const margin = (ts.margin || 0) * scale;
    const columns = ts.columns || 1;
    const tileCount = ts.tile_count || 0;

    tilesetCtx.strokeStyle = 'rgba(255,255,255,0.15)';
    tilesetCtx.lineWidth = 1;
    for (let i = 0; i < tileCount; i++) {
        const col = i % columns;
        const row = Math.floor(i / columns);
        const x = col * (tw + spacing) + margin;
        const y = row * (th + spacing) + margin;
        tilesetCtx.strokeRect(x + 0.5, y + 0.5, tw - 1, th - 1);
    }

    // Highlight selected tile
    if (selectedTile && selectedTile.tilesetIndex === activeTilesetIndex) {
        const col = selectedTile.localIndex % columns;
        const row = Math.floor(selectedTile.localIndex / columns);
        const x = col * (tw + spacing) + margin;
        const y = row * (th + spacing) + margin;
        tilesetCtx.strokeStyle = '#58a6ff';
        tilesetCtx.lineWidth = 2;
        tilesetCtx.strokeRect(x + 1, y + 1, tw - 2, th - 2);
    }
}

function handleTilesetClick(e) {
    if (!mapData || !mapData.tilesets) return;
    const ts = mapData.tilesets[activeTilesetIndex];
    if (!ts) return;

    const rect = tilesetCanvas.getBoundingClientRect();
    const scale = Math.max(1, Math.floor(64 / (ts.tile_width || 16)));
    const tw = (ts.tile_width || 16) * scale;
    const th = (ts.tile_height || 16) * scale;
    const spacing = (ts.spacing || 0) * scale;
    const margin = (ts.margin || 0) * scale;
    const columns = ts.columns || 1;

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const col = Math.floor((x - margin) / (tw + spacing));
    const row = Math.floor((y - margin) / (th + spacing));
    const localIndex = row * columns + col;

    if (localIndex < 0 || localIndex >= (ts.tile_count || 0)) return;

    const firstGid = ts.first_gid || ts.firstgid || 1;
    selectedTile = {
        gid: firstGid + localIndex,
        tilesetIndex: activeTilesetIndex,
        localIndex: localIndex,
    };

    renderTilesetCanvas(); // redraw to show selection highlight
}

// ============================================================
// Tile Painting
// ============================================================
function paintTileAt(e) {
    const rect = canvas.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const { mapX, mapY } = screenToMap(screenX, screenY);

    const grid = mapData.grid || {};
    const tileWidth = grid.tile_width || 16;
    const tileHeight = grid.tile_height || 16;
    const col = Math.floor(mapX / tileWidth);
    const row = Math.floor(mapY / tileHeight);

    if (col < 0 || row < 0 || col >= (grid.width || 0) || row >= (grid.height || 0)) return;

    // Find the active tile layer
    const activeLayer = findActiveTileLayer();
    if (!activeLayer) return;

    const layerWidth = activeLayer.width || grid.width || 0;
    const idx = row * layerWidth + col;

    if (activeLayer.data[idx] === selectedTile.gid) return; // no change

    // Push undo entry before modifying
    const layerIndex = mapData.layers.indexOf(activeLayer);
    pushUndo(layerIndex, idx, activeLayer.data[idx]);

    activeLayer.data[idx] = selectedTile.gid;
    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    render();
}

function eraseTileAt(e) {
    const rect = canvas.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const { mapX, mapY } = screenToMap(screenX, screenY);

    const grid = mapData.grid || {};
    const tileWidth = grid.tile_width || 16;
    const tileHeight = grid.tile_height || 16;
    const col = Math.floor(mapX / tileWidth);
    const row = Math.floor(mapY / tileHeight);

    if (col < 0 || row < 0 || col >= (grid.width || 0) || row >= (grid.height || 0)) return;

    const activeLayer = findActiveTileLayer();
    if (!activeLayer) return;

    const layerWidth = activeLayer.width || grid.width || 0;
    const idx = row * layerWidth + col;

    if (activeLayer.data[idx] === 0) return;

    // Push undo entry before modifying
    const layerIndex = mapData.layers.indexOf(activeLayer);
    pushUndo(layerIndex, idx, activeLayer.data[idx]);

    activeLayer.data[idx] = 0;
    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    render();
}

function findActiveTileLayer() {
    if (!mapData || !mapData.layers) return null;

    // Use the selected layer if it's a visible tile layer
    if (selectedLayerIndex >= 0 && selectedLayerIndex < mapData.layers.length) {
        const layer = mapData.layers[selectedLayerIndex];
        const isTile = layer.type === 'tile' || layer.layer_type === 'tile';
        if (isTile && layerVisibility[layer.name]) {
            return layer;
        }
    }

    // Fallback: first visible tile layer
    for (const layer of mapData.layers) {
        const isTile = layer.type === 'tile' || layer.layer_type === 'tile';
        if (isTile && layerVisibility[layer.name]) {
            return layer;
        }
    }
    return null;
}

// ============================================================
// Undo / Redo
// ============================================================
function pushUndo(layerIndex, idx, oldValue) {
    undoStack.push({ layerIndex, idx, oldValue });
    if (undoStack.length > MAX_UNDO) {
        undoStack.shift();
    }
    // Clear redo stack on new action
    redoStack = [];
}

function undo() {
    if (undoStack.length === 0 || !mapData || !mapData.layers) return;
    const entry = undoStack.pop();
    const layer = mapData.layers[entry.layerIndex];
    if (!layer || !layer.data) return;

    // Save current value for redo
    const currentValue = layer.data[entry.idx];
    redoStack.push({ layerIndex: entry.layerIndex, idx: entry.idx, oldValue: currentValue });

    // Restore old value
    layer.data[entry.idx] = entry.oldValue;
    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    render();
}

function redo() {
    if (redoStack.length === 0 || !mapData || !mapData.layers) return;
    const entry = redoStack.pop();
    const layer = mapData.layers[entry.layerIndex];
    if (!layer || !layer.data) return;

    // Save current value for undo
    const currentValue = layer.data[entry.idx];
    undoStack.push({ layerIndex: entry.layerIndex, idx: entry.idx, oldValue: currentValue });

    // Apply redo value
    layer.data[entry.idx] = entry.oldValue;
    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    render();
}

// ============================================================
// Layer Management
// ============================================================
function addLayer() {
    if (!mapData) return;
    const name = prompt('Layer name:');
    if (!name || !name.trim()) return;

    const grid = mapData.grid || {};
    const size = (grid.width || 0) * (grid.height || 0);
    const newLayer = {
        type: 'tile',
        name: name.trim(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: 'dense',
        data: new Array(size).fill(0),
    };
    mapData.layers.push(newLayer);
    layerVisibility[newLayer.name] = true;
    selectedLayerIndex = mapData.layers.length - 1;
    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    updateLayerList();
    render();
}

function deleteLayer() {
    if (!mapData || !mapData.layers) return;
    if (selectedLayerIndex < 0 || selectedLayerIndex >= mapData.layers.length) return;

    const layer = mapData.layers[selectedLayerIndex];
    if (!confirm('Delete layer "' + layer.name + '"?')) return;

    mapData.layers.splice(selectedLayerIndex, 1);
    delete layerVisibility[layer.name];

    // Adjust selected index
    if (selectedLayerIndex >= mapData.layers.length) {
        selectedLayerIndex = Math.max(0, mapData.layers.length - 1);
    }

    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    updateLayerList();
    updateMapInfo();
    render();
}

function moveLayerUp() {
    if (!mapData || !mapData.layers) return;
    if (selectedLayerIndex <= 0) return;

    const layers = mapData.layers;
    const temp = layers[selectedLayerIndex - 1];
    layers[selectedLayerIndex - 1] = layers[selectedLayerIndex];
    layers[selectedLayerIndex] = temp;
    selectedLayerIndex--;

    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    updateLayerList();
    render();
}

function moveLayerDown() {
    if (!mapData || !mapData.layers) return;
    if (selectedLayerIndex >= mapData.layers.length - 1) return;

    const layers = mapData.layers;
    const temp = layers[selectedLayerIndex + 1];
    layers[selectedLayerIndex + 1] = layers[selectedLayerIndex];
    layers[selectedLayerIndex] = temp;
    selectedLayerIndex++;

    isDirty = true;
    document.getElementById('btn-save').disabled = false;
    updateLayerList();
    render();
}

// ============================================================
// Save
// ============================================================
function saveMap() {
    if (!mapData) return;
    const json = JSON.stringify(mapData, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = (mapData.name || 'map') + '.cartile';
    a.click();
    URL.revokeObjectURL(url);
    isDirty = false;
}

// ============================================================
// File Loading
// ============================================================
function handleFiles(fileList) {
    const files = Array.from(fileList);
    const mapFiles = [];
    const imageFiles = [];

    for (const f of files) {
        const name = f.name.toLowerCase();
        if (name.endsWith('.cartile') || name.endsWith('.json')) {
            mapFiles.push(f);
        } else if (name.endsWith('.png') || name.endsWith('.jpg') || name.endsWith('.jpeg') || name.endsWith('.webp')) {
            imageFiles.push(f);
        }
    }

    // Load images first, then map
    const imagePromises = imageFiles.map(f => loadImageFile(f));
    const mapPromise = mapFiles.length > 0 ? loadMapFile(mapFiles[0]) : Promise.resolve(null);

    Promise.all([...imagePromises, mapPromise])
        .then(results => {
            const map = results[results.length - 1];
            if (map) {
                loadMap(map);
            } else if (mapFiles.length === 0 && imageFiles.length > 0) {
                showToast('No map file found. Drop a .cartile or Tiled .json file along with images.');
            }
        })
        .catch(err => {
            showToast('Error loading files: ' + err.message);
        });
}

async function loadMapFile(file) {
    const text = await file.text();
    const name = file.name.toLowerCase();

    if (name.endsWith('.cartile')) {
        if (!wasmReady) {
            // Attempt direct JSON parse as fallback
            try {
                return JSON.parse(text);
            } catch {
                throw new Error('WASM not available and file is not valid JSON');
            }
        }
        const mapJson = wasmModule.parseCartileMap(text);
        return JSON.parse(mapJson);
    }

    if (name.endsWith('.json')) {
        if (!wasmReady) {
            throw new Error('WASM not available — cannot convert Tiled JSON. Build the WASM package first.');
        }
        const resultJson = wasmModule.convertTiledJson(text, file.name.replace(/\.json$/i, ''));
        const result = JSON.parse(resultJson);

        // Show conversion warnings if any
        if (result.warnings && result.warnings.length > 0) {
            const warnEl = document.getElementById('status-warnings');
            warnEl.textContent = '⚠ ' + result.warnings.join(' | ');
        }

        return JSON.parse(result.cartile_json);
    }

    throw new Error('Unsupported map file format: ' + file.name);
}

function loadImageFile(file) {
    return new Promise((resolve, reject) => {
        const url = URL.createObjectURL(file);
        const img = new Image();
        img.onload = () => {
            // Store by basename (lowercase)
            const basename = file.name.split('/').pop().toLowerCase();
            tilesetImages[basename] = img;
            resolve();
        };
        img.onerror = () => reject(new Error('Failed to load image: ' + file.name));
        img.src = url;
    });
}

function loadMap(map) {
    mapData = map;
    hoveredTile = null;
    isDirty = false;
    document.getElementById('btn-save').disabled = true;

    console.log('Map loaded:', map.name, 'grid:', map.grid);
    console.log('Loaded tileset images:', Object.keys(tilesetImages));
    if (map.tilesets) {
        for (const ts of map.tilesets) {
            const img = ts.image || '';
            const basename = img.split('/').pop().toLowerCase();
            console.log('Tileset', ts.name, 'needs image:', basename, 'found:', !!tilesetImages[basename]);
        }
    }

    // Initialize layer visibility and select first layer
    layerVisibility = {};
    selectedLayerIndex = 0;
    if (map.layers) {
        for (const layer of map.layers) {
            layerVisibility[layer.name] = layer.visible !== false;
        }
    }

    // Clear undo/redo stacks on new map load
    undoStack = [];
    redoStack = [];

    // Hide welcome overlay
    document.getElementById('welcome-overlay').classList.add('hidden');

    // Update UI
    updateLayerList();
    updateMapInfo();
    updateTileProperties(null);

    // Center the map in the canvas
    const grid = map.grid || {};
    const mapPixelW = (grid.width || 0) * (grid.tile_width || 16);
    const mapPixelH = (grid.height || 0) * (grid.tile_height || 16);
    const container = document.getElementById('canvas-container');
    camera.zoom = 1.0;
    camera.x = -(container.clientWidth / camera.zoom - mapPixelW) / 2;
    camera.y = -(container.clientHeight / camera.zoom - mapPixelH) / 2;
    updateZoomDisplay();

    // Reset paint state
    activeTilesetIndex = 0;
    selectedTile = null;

    // If already in paint mode, show tileset panel
    if (currentMode === 'paint') {
        document.getElementById('tileset-panel').classList.remove('hidden');
        renderTilesetPanel();
    }

    render();
}

// ============================================================
// Canvas Rendering
// ============================================================
function render() {
    if (!canvas || !ctx) return;

    const w = canvas.width;
    const h = canvas.height;

    // Clear
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#0d1117';
    ctx.fillRect(0, 0, w, h);

    if (!mapData) return;

    // Apply camera transform
    ctx.setTransform(camera.zoom, 0, 0, camera.zoom, -camera.x * camera.zoom, -camera.y * camera.zoom);

    const grid = mapData.grid || {};
    const tileWidth = grid.tile_width || 16;
    const tileHeight = grid.tile_height || 16;
    const mapWidth = grid.width || 0;
    const mapHeight = grid.height || 0;

    // Render each visible tile layer in order
    if (!mapData.layers) return;

    for (const layer of mapData.layers) {
        if (layer.type !== 'tile' && layer.layer_type !== 'tile') continue;
        if (!layerVisibility[layer.name]) continue;

        const data = layer.data || layer.tiles;
        if (!data) continue;

        const layerWidth = layer.width || mapWidth;
        const layerHeight = layer.height || mapHeight;

        for (let row = 0; row < layerHeight; row++) {
            for (let col = 0; col < layerWidth; col++) {
                const idx = row * layerWidth + col;
                const raw = data[idx];
                if (!raw || raw === 0) continue;

                const tileInfo = extractTileInfo(raw);
                if (tileInfo.gid === 0) continue;

                const tsr = findTilesetForGid(tileInfo.gid);
                if (!tsr) continue;

                const destX = col * tileWidth;
                const destY = row * tileHeight;

                const tw = tsr.tileset.tile_width || tileWidth;
                const th = tsr.tileset.tile_height || tileHeight;
                const spacing = tsr.tileset.spacing || 0;
                const margin = tsr.tileset.margin || 0;
                const columns = tsr.tileset.columns || 1;

                const localCol = tsr.localIndex % columns;
                const localRow = Math.floor(tsr.localIndex / columns);
                const sx = localCol * (tw + spacing) + margin;
                const sy = localRow * (th + spacing) + margin;

                if (tsr.image) {
                    renderTile(ctx, tileInfo, destX, destY, tw, th, tsr.image, sx, sy);
                } else {
                    // Placeholder for missing tileset image
                    renderPlaceholder(ctx, destX, destY, tw, th);
                }
            }
        }
    }

    // Render object layers
    renderObjectLayers();

    // Draw grid overlay (after tiles, before axis lines)
    if (showGrid) {
        renderGridOverlay();
    }

    // Draw map boundary
    const totalW = mapWidth * tileWidth;
    const totalH = mapHeight * tileHeight;
    ctx.strokeStyle = 'rgba(88, 166, 255, 0.4)';
    ctx.lineWidth = 1 / camera.zoom;
    ctx.strokeRect(0, 0, totalW, totalH);

    // Draw origin axis lines (extend beyond map)
    const axisExtent = 2000 / camera.zoom;
    ctx.lineWidth = 1 / camera.zoom;

    // X-axis (horizontal line at y=0)
    ctx.strokeStyle = 'rgba(255, 80, 80, 0.5)';
    ctx.beginPath();
    ctx.moveTo(-axisExtent, 0);
    ctx.lineTo(totalW + axisExtent, 0);
    ctx.stroke();

    // Y-axis (vertical line at x=0)
    ctx.strokeStyle = 'rgba(80, 255, 80, 0.5)';
    ctx.beginPath();
    ctx.moveTo(0, -axisExtent);
    ctx.lineTo(0, totalH + axisExtent);
    ctx.stroke();

    // Axis labels (drawn in screen space)
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    const originScreen = mapToScreen(0, 0);
    ctx.font = '10px sans-serif';
    if (originScreen.x > 10 && originScreen.x < w - 10 &&
        originScreen.y > 10 && originScreen.y < h - 10) {
        ctx.fillStyle = 'rgba(255, 80, 80, 0.7)';
        ctx.fillText('X', originScreen.x + 12, originScreen.y - 4);
        ctx.fillStyle = 'rgba(80, 255, 80, 0.7)';
        ctx.fillText('Y', originScreen.x - 12, originScreen.y + 14);
    }
    ctx.restore();
}

// ============================================================
// Object Layer Rendering
// ============================================================

function renderObjectLayers() {
    if (!mapData || !mapData.layers) return;

    for (const layer of mapData.layers) {
        if (layer.type !== 'object') continue;
        if (!layerVisibility[layer.name]) continue;

        const objects = layer.objects || [];

        for (const obj of objects) {
            renderObject(obj);
        }
    }
}

function renderObject(obj) {
    const shape = obj.shape || 'rect';
    const x = obj.x || 0;
    const y = obj.y || 0;
    const w = obj.width || 0;
    const h = obj.height || 0;
    const rotation = obj.rotation || 0;

    ctx.save();

    // Apply rotation around (x, y) if needed
    if (rotation !== 0) {
        ctx.translate(x, y);
        ctx.rotate(rotation * Math.PI / 180);
        ctx.translate(-x, -y);
    }

    const fillColor = 'rgba(88, 166, 255, 0.15)';
    const strokeColor = 'rgba(88, 166, 255, 0.7)';
    const lineWidth = 1.5 / camera.zoom;

    ctx.fillStyle = fillColor;
    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = lineWidth;

    switch (shape) {
        case 'point': {
            const r = 4 / camera.zoom;
            ctx.fillStyle = '#58a6ff';
            ctx.beginPath();
            ctx.arc(x, y, r, 0, Math.PI * 2);
            ctx.fill();
            // Draw crosshair
            ctx.strokeStyle = '#58a6ff';
            ctx.lineWidth = 1 / camera.zoom;
            const cr = 8 / camera.zoom;
            ctx.beginPath();
            ctx.moveTo(x - cr, y); ctx.lineTo(x + cr, y);
            ctx.moveTo(x, y - cr); ctx.lineTo(x, y + cr);
            ctx.stroke();
            break;
        }
        case 'rect': {
            ctx.fillRect(x, y, w, h);
            ctx.strokeRect(x, y, w, h);
            break;
        }
        case 'ellipse': {
            ctx.beginPath();
            ctx.ellipse(x + w / 2, y + h / 2, w / 2, h / 2, 0, 0, Math.PI * 2);
            ctx.fill();
            ctx.stroke();
            break;
        }
        case 'polygon': {
            if (!obj.points || obj.points.length < 3) break;
            ctx.beginPath();
            ctx.moveTo(x + obj.points[0].x, y + obj.points[0].y);
            for (let i = 1; i < obj.points.length; i++) {
                ctx.lineTo(x + obj.points[i].x, y + obj.points[i].y);
            }
            ctx.closePath();
            ctx.fill();
            ctx.stroke();
            break;
        }
        case 'polyline': {
            if (!obj.points || obj.points.length < 2) break;
            ctx.beginPath();
            ctx.moveTo(x + obj.points[0].x, y + obj.points[0].y);
            for (let i = 1; i < obj.points.length; i++) {
                ctx.lineTo(x + obj.points[i].x, y + obj.points[i].y);
            }
            ctx.stroke();
            break;
        }
    }

    // Draw object name label
    if (obj.name) {
        const fontSize = Math.max(8, 11 / camera.zoom);
        ctx.font = fontSize + 'px sans-serif';
        ctx.fillStyle = '#58a6ff';
        ctx.textBaseline = 'bottom';

        if (shape === 'point') {
            ctx.fillText(obj.name, x + 10 / camera.zoom, y - 4 / camera.zoom);
        } else {
            ctx.fillText(obj.name, x + 2 / camera.zoom, y - 2 / camera.zoom);
        }
    }

    ctx.restore();
}

// ============================================================
// Grid Overlay
// ============================================================
function renderGridOverlay() {
    const grid = mapData.grid || {};
    const tw = grid.tile_width || 16;
    const th = grid.tile_height || 16;
    const mw = grid.width || 0;
    const mh = grid.height || 0;

    ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
    ctx.lineWidth = 1 / camera.zoom;

    for (let x = 0; x <= mw; x++) {
        ctx.beginPath();
        ctx.moveTo(x * tw, 0);
        ctx.lineTo(x * tw, mh * th);
        ctx.stroke();
    }
    for (let y = 0; y <= mh; y++) {
        ctx.beginPath();
        ctx.moveTo(0, y * th);
        ctx.lineTo(mw * tw, y * th);
        ctx.stroke();
    }
}

function mapToScreen(mapX, mapY) {
    return {
        x: (mapX - camera.x) * camera.zoom,
        y: (mapY - camera.y) * camera.zoom,
    };
}

function renderTile(ctx, tileInfo, dx, dy, tw, th, img, sx, sy) {
    const { flipH, flipV, flipD } = tileInfo;
    const hasTransform = flipH || flipV || flipD;

    if (!hasTransform) {
        ctx.drawImage(img, sx, sy, tw, th, dx, dy, tw, th);
        return;
    }

    ctx.save();
    ctx.translate(dx + tw / 2, dy + th / 2);

    // Apply flip/rotation transforms
    // Diagonal flip + horizontal/vertical flips encode 0/90/180/270 rotations
    if (flipD) {
        if (flipH && flipV) {
            // 90 degrees counter-clockwise
            ctx.rotate(-Math.PI / 2);
            ctx.scale(-1, 1);
        } else if (flipH) {
            // 90 degrees clockwise
            ctx.rotate(Math.PI / 2);
        } else if (flipV) {
            // 90 degrees counter-clockwise
            ctx.rotate(-Math.PI / 2);
        } else {
            // Diagonal flip only: rotate 90 + flip horizontal
            ctx.rotate(Math.PI / 2);
            ctx.scale(-1, 1);
        }
    } else {
        let scaleX = flipH ? -1 : 1;
        let scaleY = flipV ? -1 : 1;
        ctx.scale(scaleX, scaleY);
    }

    ctx.drawImage(img, sx, sy, tw, th, -tw / 2, -th / 2, tw, th);
    ctx.restore();
}

function renderPlaceholder(ctx, dx, dy, tw, th) {
    ctx.fillStyle = '#3d1f5c';
    ctx.fillRect(dx, dy, tw, th);
    ctx.fillStyle = '#c9a0ff';
    ctx.font = Math.min(tw, th) * 0.6 + 'px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('?', dx + tw / 2, dy + th / 2);
}

function extractTileInfo(raw) {
    // Ensure unsigned 32-bit treatment
    const u = raw >>> 0;
    return {
        gid: u & GID_MASK,
        flipH: (u & FLIP_H) !== 0,
        flipV: (u & FLIP_V) !== 0,
        flipD: (u & FLIP_D) !== 0,
    };
}

function findTilesetForGid(gid) {
    if (!mapData || !mapData.tilesets) return null;

    // Find the tileset with the highest first_gid <= gid
    let match = null;
    for (const ts of mapData.tilesets) {
        const firstGid = ts.first_gid || ts.firstgid || 1;
        if (gid >= firstGid) {
            if (!match || firstGid > (match.first_gid || match.firstgid || 1)) {
                match = ts;
            }
        }
    }

    if (!match) return null;

    const firstGid = match.first_gid || match.firstgid || 1;
    const localIndex = gid - firstGid;

    // Find image by basename
    const imagePath = match.image || '';
    const basename = imagePath.split('/').pop().toLowerCase();
    const image = tilesetImages[basename] || null;

    return {
        tileset: match,
        localIndex,
        image,
    };
}

// ============================================================
// Camera (Pan / Zoom)
// ============================================================
function handleMouseDown(e) {
    // Paint mode: left click paints, right click erases
    if (currentMode === 'paint' && mapData) {
        if (e.button === 0 && selectedTile) {
            paintTileAt(e);
            return;
        }
        if (e.button === 2) {
            eraseTileAt(e);
            return;
        }
    }

    // Pan with left click in view mode, or middle mouse in any mode
    if ((e.button === 0 && currentMode === 'view') || e.button === 1) {
        isPanning = true;
        panStart.x = e.clientX;
        panStart.y = e.clientY;
        canvas.style.cursor = 'grabbing';
    }
}

function handleMouseMove(e) {
    // Paint on drag (left button) or erase on drag (right button)
    if (currentMode === 'paint' && mapData && !isPanning) {
        if (e.buttons === 1 && selectedTile) {
            paintTileAt(e);
        } else if (e.buttons === 2) {
            eraseTileAt(e);
        }
    }

    if (isPanning) {
        const dx = e.clientX - panStart.x;
        const dy = e.clientY - panStart.y;
        camera.x -= dx / camera.zoom;
        camera.y -= dy / camera.zoom;
        panStart.x = e.clientX;
        panStart.y = e.clientY;
        render();
    }

    // Hover / tile info
    if (mapData) {
        const rect = canvas.getBoundingClientRect();
        const screenX = e.clientX - rect.left;
        const screenY = e.clientY - rect.top;
        updateHoveredTile(screenX, screenY);
    }
}

function handleMouseUp() {
    isPanning = false;
    if (currentMode === 'paint') {
        canvas.style.cursor = 'crosshair';
    } else {
        canvas.style.cursor = 'default';
    }
}

function handleWheel(e) {
    e.preventDefault();

    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Map position under cursor before zoom
    const mapXBefore = camera.x + mouseX / camera.zoom;
    const mapYBefore = camera.y + mouseY / camera.zoom;

    // Adjust zoom
    const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
    camera.zoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, camera.zoom + delta));

    // Adjust camera so the point under cursor stays fixed
    camera.x = mapXBefore - mouseX / camera.zoom;
    camera.y = mapYBefore - mouseY / camera.zoom;

    updateZoomDisplay();
    render();
}

function updateZoomDisplay() {
    document.getElementById('zoom-display').textContent = Math.round(camera.zoom * 100) + '%';
}

// ============================================================
// Hover / Tile Info
// ============================================================
function screenToMap(screenX, screenY) {
    const mapX = camera.x + screenX / camera.zoom;
    const mapY = camera.y + screenY / camera.zoom;
    return { mapX, mapY };
}

function updateHoveredTile(screenX, screenY) {
    const { mapX, mapY } = screenToMap(screenX, screenY);
    const grid = mapData.grid || {};
    const tileWidth = grid.tile_width || 16;
    const tileHeight = grid.tile_height || 16;
    const mapWidth = grid.width || 0;
    const mapHeight = grid.height || 0;

    const col = Math.floor(mapX / tileWidth);
    const row = Math.floor(mapY / tileHeight);

    if (col < 0 || row < 0 || col >= mapWidth || row >= mapHeight) {
        document.getElementById('status-cursor').textContent = '–';
        document.getElementById('status-gid').textContent = '–';
        document.getElementById('status-layer').textContent = '–';
        updateTileProperties(null);
        return;
    }

    document.getElementById('status-cursor').textContent = 'Cursor: (' + col + ', ' + row + ')';

    // Find topmost visible tile at position
    let foundGid = 0;
    let foundLayerName = '–';

    if (mapData.layers) {
        for (let i = mapData.layers.length - 1; i >= 0; i--) {
            const layer = mapData.layers[i];
            if (layer.type !== 'tile' && layer.layer_type !== 'tile') continue;
            if (!layerVisibility[layer.name]) continue;

            const data = layer.data || layer.tiles;
            if (!data) continue;

            const layerWidth = layer.width || mapWidth;
            const idx = row * layerWidth + col;
            const raw = data[idx];

            if (raw && raw !== 0) {
                const info = extractTileInfo(raw);
                if (info.gid !== 0) {
                    foundGid = info.gid;
                    foundLayerName = layer.name;
                    break;
                }
            }
        }
    }

    document.getElementById('status-gid').textContent = foundGid ? 'GID: ' + foundGid : '–';
    document.getElementById('status-layer').textContent = foundLayerName !== '–' ? 'Layer: ' + foundLayerName : '–';

    // Look up tile properties
    if (foundGid) {
        const tsr = findTilesetForGid(foundGid);
        if (tsr) {
            const tile = {
                col, row,
                gid: foundGid,
                tilesetName: tsr.tileset.name || '(unnamed)',
                localIndex: tsr.localIndex,
                properties: getTileProperties(tsr.tileset, tsr.localIndex),
            };
            updateTileProperties(tile);
        } else {
            updateTileProperties(null);
        }
    } else {
        updateTileProperties(null);
    }
}

function getTileProperties(tileset, localIndex) {
    if (!tileset.tiles) return {};

    // tiles could be an array or an object keyed by index
    if (Array.isArray(tileset.tiles)) {
        const entry = tileset.tiles.find(t => t.id === localIndex || t.local_id === localIndex);
        return entry?.properties || {};
    }

    // Object keyed by string index
    const entry = tileset.tiles[String(localIndex)];
    return entry?.properties || {};
}

// ============================================================
// UI Updates
// ============================================================
function updateLayerList() {
    const container = document.getElementById('layer-list');
    container.innerHTML = '';

    if (!mapData || !mapData.layers) return;

    for (let i = 0; i < mapData.layers.length; i++) {
        const layer = mapData.layers[i];
        const layerIndex = i;

        const item = document.createElement('div');
        item.className = 'layer-item'
            + (layerVisibility[layer.name] ? '' : ' hidden-layer')
            + (i === selectedLayerIndex ? ' active-layer' : '');

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = layerVisibility[layer.name] !== false;

        const isTileLayer = layer.type === 'tile' || layer.layer_type === 'tile';

        checkbox.addEventListener('change', (e) => {
            e.stopPropagation();
            layerVisibility[layer.name] = checkbox.checked;
            item.classList.toggle('hidden-layer', !checkbox.checked);
            render();
        });

        // Prevent checkbox click from triggering layer selection
        checkbox.addEventListener('click', (e) => {
            e.stopPropagation();
        });

        const nameSpan = document.createElement('span');
        nameSpan.className = 'layer-name';
        nameSpan.textContent = layer.name;

        item.appendChild(checkbox);
        item.appendChild(nameSpan);

        if (!isTileLayer) {
            const typeSpan = document.createElement('span');
            typeSpan.className = 'layer-type';
            typeSpan.textContent = ' (' + (layer.type || layer.layer_type || 'unknown') + ')';
            item.appendChild(typeSpan);
        }

        // Click to select active layer
        item.addEventListener('click', () => {
            selectedLayerIndex = layerIndex;
            updateLayerList();
        });

        container.appendChild(item);
    }
}

function updateMapInfo() {
    const container = document.getElementById('map-info');
    if (!mapData) {
        container.innerHTML = '<span class="hint">No map loaded</span>';
        return;
    }

    const grid = mapData.grid || {};
    const lines = [
        { label: 'Name', value: mapData.name || '(unnamed)' },
        { label: 'Size', value: (grid.width || '?') + ' × ' + (grid.height || '?') + ' tiles' },
        { label: 'Tile Size', value: (grid.tile_width || '?') + ' × ' + (grid.tile_height || '?') + ' px' },
        { label: 'Layers', value: mapData.layers ? mapData.layers.length : 0 },
        { label: 'Tilesets', value: mapData.tilesets ? mapData.tilesets.length : 0 },
    ];

    if (mapData.cartile) {
        lines.push({ label: 'Format', value: 'v' + mapData.cartile });
    }

    container.innerHTML = lines
        .map(l => '<div><span class="info-label">' + l.label + ':</span> ' + l.value + '</div>')
        .join('');
}

function updateTileProperties(tile) {
    const container = document.getElementById('tile-props');

    if (!tile) {
        container.innerHTML = '<span class="hint">Hover over a tile</span>';
        return;
    }

    let html = '<div><span class="prop-key">Position:</span> <span class="prop-value">(' + tile.col + ', ' + tile.row + ')</span></div>';
    html += '<div><span class="prop-key">GID:</span> <span class="prop-value">' + tile.gid + '</span></div>';
    html += '<div><span class="prop-key">Tileset:</span> <span class="prop-value">' + tile.tilesetName + '</span></div>';
    html += '<div><span class="prop-key">Local ID:</span> <span class="prop-value">' + tile.localIndex + '</span></div>';

    if (tile.properties && Object.keys(tile.properties).length > 0) {
        html += '<hr style="border-color: var(--border); margin: 6px 0;">';
        for (const [key, value] of Object.entries(tile.properties)) {
            html += '<div><span class="prop-key">' + key + ':</span> <span class="prop-value">' + value + '</span></div>';
        }
    }

    container.innerHTML = html;
}

function showToast(msg) {
    const toast = document.getElementById('toast');
    toast.textContent = msg;
    toast.classList.remove('hidden');

    setTimeout(() => {
        toast.classList.add('hidden');
    }, 5000);
}

// ============================================================
// Canvas Resize
// ============================================================
function resizeCanvas() {
    const container = document.getElementById('canvas-container');
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;

    // Disable image smoothing for crisp pixel art
    ctx.imageSmoothingEnabled = false;

    render();
}

// ============================================================
// New Map Modal
// ============================================================
function showNewMapModal() {
    document.getElementById('new-map-modal').classList.remove('hidden');
    document.getElementById('new-map-name').focus();
    document.getElementById('new-map-name').select();
    pendingTilesetFile = null;
    document.getElementById('new-tileset-label').textContent = 'Drop a PNG here or click to select';
    document.getElementById('new-tileset-drop').classList.remove('has-file');
}

function hideNewMapModal() {
    document.getElementById('new-map-modal').classList.add('hidden');
    pendingTilesetFile = null;
}

function isNewMapModalOpen() {
    return !document.getElementById('new-map-modal').classList.contains('hidden');
}

function createNewMap() {
    const name = document.getElementById('new-map-name').value.trim() || 'untitled';
    const mapWidth = parseInt(document.getElementById('new-map-width').value) || 20;
    const mapHeight = parseInt(document.getElementById('new-map-height').value) || 15;
    const tileWidth = parseInt(document.getElementById('new-tile-width').value) || 16;
    const tileHeight = parseInt(document.getElementById('new-tile-height').value) || 16;

    const size = mapWidth * mapHeight;

    const map = {
        cartile: '0.1.0',
        type: 'map',
        name: name,
        grid: {
            type: 'square',
            width: mapWidth,
            height: mapHeight,
            tile_width: tileWidth,
            tile_height: tileHeight,
            topology: 'bounded',
            projection: { type: 'orthogonal' },
            height_mode: 'none',
        },
        tilesets: [],
        layers: [
            {
                type: 'tile',
                name: 'Layer 1',
                visible: true,
                opacity: 1.0,
                elevation: 0,
                encoding: 'dense',
                data: new Array(size).fill(0),
            }
        ],
    };

    // If a tileset image was provided, create a tileset entry
    if (pendingTilesetFile) {
        const img = tilesetImages[pendingTilesetFile.name.toLowerCase()];
        if (img) {
            const columns = Math.floor(img.width / tileWidth);
            const rows = Math.floor(img.height / tileHeight);
            map.tilesets.push({
                name: pendingTilesetFile.name.replace(/\.[^.]+$/, ''),
                tile_width: tileWidth,
                tile_height: tileHeight,
                image: pendingTilesetFile.name,
                image_width: img.width,
                image_height: img.height,
                columns: columns,
                tile_count: columns * rows,
                margin: 0,
                spacing: 0,
                first_gid: 1,
            });
        }
    }

    hideNewMapModal();
    loadMap(map);
}

function handleNewTilesetFile(file) {
    const name = file.name.toLowerCase();
    if (!name.endsWith('.png') && !name.endsWith('.jpg') && !name.endsWith('.webp')) {
        showToast('Please select an image file (PNG, JPG, WEBP)');
        return;
    }

    pendingTilesetFile = file;

    // Load the image into tilesetImages immediately
    const url = URL.createObjectURL(file);
    const img = new Image();
    img.onload = () => {
        tilesetImages[file.name.toLowerCase()] = img;
        document.getElementById('new-tileset-label').textContent = '\u2713 ' + file.name + ' (' + img.width + '\u00d7' + img.height + ')';
        document.getElementById('new-tileset-drop').classList.add('has-file');
    };
    img.onerror = () => {
        showToast('Failed to load image: ' + file.name);
        pendingTilesetFile = null;
    };
    img.src = url;
}

// ============================================================
// Keyboard Shortcuts
// ============================================================
function handleKeyDown(e) {
    // Skip if user is typing in an input field
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.tagName === 'SELECT') {
        return;
    }

    // Ctrl/Cmd shortcuts
    if (e.ctrlKey || e.metaKey) {
        if (e.key === 'z' && !e.shiftKey) {
            e.preventDefault();
            undo();
            return;
        }
        if (e.key === 'y' || (e.key === 'z' && e.shiftKey) || (e.key === 'Z' && e.shiftKey)) {
            e.preventDefault();
            redo();
            return;
        }
    }

    // Close modal or help on Escape
    if (e.key === 'Escape') {
        if (isNewMapModalOpen()) {
            hideNewMapModal();
            return;
        }
        if (helpVisible) {
            hideHelp();
            return;
        }
    }

    // Skip remaining shortcuts when modal is open
    if (isNewMapModalOpen()) return;

    // Single-key shortcuts (only when no modifier)
    if (e.ctrlKey || e.metaKey || e.altKey) return;

    switch (e.key.toLowerCase()) {
        case 'n':
            showNewMapModal();
            break;
        case 'v':
            setMode('view');
            break;
        case 'b':
            setMode('paint');
            break;
        case 's':
            e.preventDefault();
            if (mapData && isDirty) saveMap();
            break;
        case 'o':
            e.preventDefault();
            document.getElementById('file-input').click();
            break;
        case 'g':
            showGrid = !showGrid;
            document.getElementById('btn-grid').classList.toggle('active', showGrid);
            render();
            break;
        case '?':
            toggleHelp();
            break;
    }
}

// ============================================================
// Event Listeners
// ============================================================
function setupEventListeners() {
    const container = document.getElementById('canvas-container');

    // Drag and drop
    container.addEventListener('dragover', (e) => {
        e.preventDefault();
        container.classList.add('drag-over');
    });

    container.addEventListener('dragleave', () => {
        container.classList.remove('drag-over');
    });

    container.addEventListener('drop', (e) => {
        e.preventDefault();
        container.classList.remove('drag-over');
        if (e.dataTransfer.files.length > 0) {
            handleFiles(e.dataTransfer.files);
        }
    });

    // File input
    const fileInput = document.getElementById('file-input');
    document.getElementById('btn-open').addEventListener('click', () => {
        fileInput.click();
    });
    fileInput.addEventListener('change', () => {
        if (fileInput.files.length > 0) {
            handleFiles(fileInput.files);
        }
        // Reset so the same files can be selected again
        fileInput.value = '';
    });

    // Canvas mouse events
    canvas.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    canvas.addEventListener('wheel', handleWheel, { passive: false });
    canvas.addEventListener('contextmenu', (e) => {
        if (currentMode === 'paint') e.preventDefault();
    });

    // Resize
    window.addEventListener('resize', resizeCanvas);

    // Mode toggle
    document.getElementById('btn-mode-view').addEventListener('click', () => setMode('view'));
    document.getElementById('btn-mode-paint').addEventListener('click', () => setMode('paint'));

    // Save
    document.getElementById('btn-save').addEventListener('click', saveMap);

    // Grid toggle
    document.getElementById('btn-grid').addEventListener('click', () => {
        showGrid = !showGrid;
        document.getElementById('btn-grid').classList.toggle('active', showGrid);
        render();
    });

    // Help button
    document.getElementById('btn-help').addEventListener('click', toggleHelp);

    // Help overlay: click backdrop to close
    document.getElementById('help-overlay').addEventListener('click', (e) => {
        if (e.target === document.getElementById('help-overlay')) {
            hideHelp();
        }
    });

    // New map button
    document.getElementById('btn-new').addEventListener('click', showNewMapModal);

    // New map modal
    document.getElementById('btn-new-cancel').addEventListener('click', hideNewMapModal);
    document.querySelector('#new-map-modal .modal-overlay').addEventListener('click', hideNewMapModal);
    document.getElementById('btn-new-create').addEventListener('click', createNewMap);

    // Enter key in modal submits
    document.getElementById('new-map-modal').addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            createNewMap();
        }
        if (e.key === 'Escape') {
            hideNewMapModal();
        }
    });

    // Tileset file drop zone in new map modal
    const tilesetDrop = document.getElementById('new-tileset-drop');
    const tilesetInput = document.getElementById('new-tileset-input');

    tilesetDrop.addEventListener('click', () => tilesetInput.click());

    tilesetDrop.addEventListener('dragover', (e) => {
        e.preventDefault();
        tilesetDrop.classList.add('drag-over');
    });

    tilesetDrop.addEventListener('dragleave', () => {
        tilesetDrop.classList.remove('drag-over');
    });

    tilesetDrop.addEventListener('drop', (e) => {
        e.preventDefault();
        e.stopPropagation();
        tilesetDrop.classList.remove('drag-over');
        if (e.dataTransfer.files.length > 0) {
            handleNewTilesetFile(e.dataTransfer.files[0]);
        }
    });

    tilesetInput.addEventListener('change', () => {
        if (tilesetInput.files.length > 0) {
            handleNewTilesetFile(tilesetInput.files[0]);
        }
        tilesetInput.value = '';
    });

    // Layer management buttons
    document.getElementById('btn-layer-add').addEventListener('click', addLayer);
    document.getElementById('btn-layer-delete').addEventListener('click', deleteLayer);
    document.getElementById('btn-layer-up').addEventListener('click', moveLayerUp);
    document.getElementById('btn-layer-down').addEventListener('click', moveLayerDown);

    // Tileset panel click
    tilesetCanvas.addEventListener('click', handleTilesetClick);

    // Tileset select change
    document.getElementById('tileset-select').addEventListener('change', (e) => {
        activeTilesetIndex = parseInt(e.target.value, 10);
        selectedTile = null;
        renderTilesetCanvas();
    });

    // Toggle tileset panel body visibility
    document.getElementById('btn-toggle-tileset').addEventListener('click', () => {
        const body = document.getElementById('tileset-panel-body');
        body.style.display = body.style.display === 'none' ? '' : 'none';
    });

    // Keyboard shortcuts
    window.addEventListener('keydown', handleKeyDown);
}

// ============================================================
// Start
// ============================================================
main();
