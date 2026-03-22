// ============================================================
// cartile viewer — Main application
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

    console.log('Map loaded:', map.name, 'grid:', map.grid);
    console.log('Loaded tileset images:', Object.keys(tilesetImages));
    if (map.tilesets) {
        for (const ts of map.tilesets) {
            const img = ts.image || '';
            const basename = img.split('/').pop().toLowerCase();
            console.log('Tileset', ts.name, 'needs image:', basename, 'found:', !!tilesetImages[basename]);
        }
    }

    // Initialize layer visibility
    layerVisibility = {};
    if (map.layers) {
        for (const layer of map.layers) {
            layerVisibility[layer.name] = layer.visible !== false;
        }
    }

    // Hide welcome overlay
    document.getElementById('welcome-overlay').classList.add('hidden');

    // Update UI
    updateLayerList();
    updateMapInfo();
    updateTileProperties(null);

    // Reset camera to center the map
    camera.x = 0;
    camera.y = 0;
    camera.zoom = 1.0;
    updateZoomDisplay();

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
    if (e.button === 0 || e.button === 1) {
        isPanning = true;
        panStart.x = e.clientX;
        panStart.y = e.clientY;
        canvas.style.cursor = 'grabbing';
    }
}

function handleMouseMove(e) {
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
    canvas.style.cursor = 'default';
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

    for (const layer of mapData.layers) {
        const item = document.createElement('label');
        item.className = 'layer-item' + (layerVisibility[layer.name] ? '' : ' hidden-layer');

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = layerVisibility[layer.name] !== false;

        const isTileLayer = layer.type === 'tile' || layer.layer_type === 'tile';

        checkbox.addEventListener('change', () => {
            layerVisibility[layer.name] = checkbox.checked;
            item.classList.toggle('hidden-layer', !checkbox.checked);
            render();
        });

        const nameSpan = document.createElement('span');
        nameSpan.textContent = layer.name;

        item.appendChild(checkbox);
        item.appendChild(nameSpan);

        if (!isTileLayer) {
            const typeSpan = document.createElement('span');
            typeSpan.className = 'layer-type';
            typeSpan.textContent = ' (' + (layer.type || layer.layer_type || 'unknown') + ')';
            item.appendChild(typeSpan);
        }

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

    // Resize
    window.addEventListener('resize', resizeCanvas);
}

// ============================================================
// Start
// ============================================================
main();
