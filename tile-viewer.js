const map = L.map('map', {
    crs: L.CRS.Simple,
    zoomSnap: 0.5,
    zoomDelta: 0.5,
    minZoom: -2,
    maxZoom: 2,
    attributionControl: false
});

const TILE_SIZE = 256;
const INITIAL_ZOOM = 0;

// Create tile layer with proper coordinate handling
const tileLayer = L.tileLayer('/img_output/{x}_{y}.png', {
    tileSize: TILE_SIZE,
    noWrap: true,
    tms: false, // Set this based on your tile orientation
    detectRetina: false,
    keepBuffer: 2,
    updateWhenIdle: false
}).addTo(map);

// Proper zoom-aware tile management
function updateTiles() {
    const zoom = map.getZoom();
    const bounds = map.getBounds();

    // Convert bounds to tile coordinates
    const topLeft = map.project(bounds.getNorthWest(), zoom).divideBy(TILE_SIZE).floor();
    const bottomRight = map.project(bounds.getSouthEast(), zoom).divideBy(TILE_SIZE).floor();

    // Calculate visible tile range
    const minX = topLeft.x;
    const maxX = bottomRight.x;
    const minY = topLeft.y;
    const maxY = bottomRight.y;

    // Remove out-of-view tiles using proper iteration
    Object.keys(tileLayer._tiles).forEach(key => {
        const tile = tileLayer._tiles[key];
        const coords = tile.coords;

        if (coords.z !== zoom ||
            coords.x < minX || coords.x > maxX ||
            coords.y < minY || coords.y > maxY) {
            tileLayer._removeTile(key);
        }
    });

    // Load new tiles with proper coordinates
    for (let x = minX; x <= maxX; x++) {
        for (let y = minY; y <= maxY; y++) {
            const coords = { x: x, y: y, z: zoom };
            const key = tileLayer._tileCoordsToKey(coords);

            if (!tileLayer._tiles[key]) {
                tileLayer._addTile(coords);
            }
        }
    }
}

// Set initial view
map.setView([0, 0], INITIAL_ZOOM);

// Update tiles on any view change
map.on('moveend zoomend', updateTiles);

// Initial load
updateTiles();