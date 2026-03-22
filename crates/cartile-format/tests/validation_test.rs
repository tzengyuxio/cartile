use cartile_format::*;

/// Build a minimal valid map (2×2, one tile layer "ground").
fn base_map() -> CartileMap {
    serde_json::from_str(
        r#"{
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
    }"#,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Basic pass
// ---------------------------------------------------------------------------

#[test]
fn valid_map_passes() {
    let map = base_map();
    assert!(map.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Layer data length
// ---------------------------------------------------------------------------

#[test]
fn invalid_data_length() {
    let mut map = base_map();
    // Replace the tile layer with one that has wrong data length (3 instead of 4).
    map.layers = vec![Layer::Tile(TileLayer {
        name: "ground".to_string(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data: vec![
            TileId::new(1, false, false, false),
            TileId::new(2, false, false, false),
            TileId::new(3, false, false, false),
        ],
        properties: Default::default(),
    })];
    let err = map.validate().unwrap_err();
    assert!(
        matches!(
            err,
            ValidationError::InvalidDataLength {
                ref name,
                actual: 3,
                expected: 4
            } if name == "ground"
        ),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Duplicate layer names
// ---------------------------------------------------------------------------

#[test]
fn duplicate_layer_name() {
    let mut map = base_map();
    let layer = TileLayer {
        name: "ground".to_string(),
        visible: true,
        opacity: 1.0,
        elevation: 0,
        encoding: "dense".to_string(),
        data: vec![
            TileId::new(1, false, false, false),
            TileId::new(2, false, false, false),
            TileId::new(3, false, false, false),
            TileId::new(4, false, false, false),
        ],
        properties: Default::default(),
    };
    map.layers = vec![Layer::Tile(layer.clone()), Layer::Tile(layer)];
    let err = map.validate().unwrap_err();
    assert!(
        matches!(err, ValidationError::DuplicateLayerName { ref name } if name == "ground"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Hex grid — missing orientation
// ---------------------------------------------------------------------------

#[test]
fn hex_missing_orientation() {
    let mut map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "hexagonal",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "stagger": { "axis": "y", "index": "odd" },
            "projection": { "type": "orthogonal" }
        },
        "layers": []
    }"#,
    )
    .unwrap();
    // Ensure orientation is absent.
    map.grid.orientation = None;
    let err = map.validate().unwrap_err();
    assert_eq!(err, ValidationError::MissingHexOrientation);
}

// ---------------------------------------------------------------------------
// Hex grid — missing stagger
// ---------------------------------------------------------------------------

#[test]
fn hex_missing_stagger() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "hexagonal",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "orientation": "pointy_top",
            "projection": { "type": "orthogonal" }
        },
        "layers": []
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert_eq!(err, ValidationError::MissingHexStagger);
}

// ---------------------------------------------------------------------------
// Stepped elevation — valid
// ---------------------------------------------------------------------------

#[test]
fn stepped_elevation_ok() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "height_mode": "stepped",
            "projection": { "type": "orthogonal" }
        },
        "layers": [{
            "type": "tile",
            "name": "ground",
            "elevation": 1,
            "data": [1, 2, 3, 4]
        }]
    }"#,
    )
    .unwrap();
    assert!(map.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Non-stepped height mode with elevation — error
// ---------------------------------------------------------------------------

#[test]
fn unexpected_elevation() {
    let map: CartileMap = serde_json::from_str(
        r#"{
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
        "layers": [{
            "type": "tile",
            "name": "ground",
            "elevation": 2,
            "data": [1, 2, 3, 4]
        }]
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert!(
        matches!(err, ValidationError::UnexpectedElevation { ref name } if name == "ground"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Vertex height mode requires heightmap layer
// ---------------------------------------------------------------------------

#[test]
fn vertex_needs_heightmap() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 2,
            "height": 2,
            "tile_width": 16,
            "tile_height": 16,
            "height_mode": "vertex",
            "projection": { "type": "orthogonal" }
        },
        "layers": [{
            "type": "tile",
            "name": "ground",
            "data": [1, 2, 3, 4]
        }]
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert_eq!(err, ValidationError::MissingHeightmapLayer);
}

// ---------------------------------------------------------------------------
// Overlapping GID ranges
// ---------------------------------------------------------------------------

#[test]
fn overlapping_gid_ranges() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
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
                "name": "ts_a",
                "tile_width": 16,
                "tile_height": 16,
                "image": "a.png",
                "image_width": 64,
                "image_height": 64,
                "columns": 4,
                "tile_count": 16,
                "first_gid": 1
            },
            {
                "name": "ts_b",
                "tile_width": 16,
                "tile_height": 16,
                "image": "b.png",
                "image_width": 64,
                "image_height": 64,
                "columns": 4,
                "tile_count": 16,
                "first_gid": 8
            }
        ],
        "layers": [{
            "type": "tile",
            "name": "ground",
            "data": [1]
        }]
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert!(
        matches!(err, ValidationError::OverlappingGidRanges { ref a, ref b } if a == "ts_a" && b == "ts_b"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Oblique projection — missing angle
// ---------------------------------------------------------------------------

#[test]
fn oblique_missing_angle() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 1,
            "height": 1,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "oblique" }
        },
        "layers": [{
            "type": "tile",
            "name": "ground",
            "data": [1]
        }]
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert_eq!(err, ValidationError::MissingObliqueAngle);
}

// ---------------------------------------------------------------------------
// Duplicate object IDs across layers
// ---------------------------------------------------------------------------

#[test]
fn duplicate_object_id() {
    let map: CartileMap = serde_json::from_str(
        r#"{
        "cartile": "0.1.0",
        "type": "map",
        "name": "test",
        "grid": {
            "type": "square",
            "width": 1,
            "height": 1,
            "tile_width": 16,
            "tile_height": 16,
            "projection": { "type": "orthogonal" }
        },
        "layers": [
            {
                "type": "object",
                "name": "entities",
                "objects": [
                    { "id": 1, "x": 0.0, "y": 0.0, "shape": "point" }
                ]
            },
            {
                "type": "object",
                "name": "triggers",
                "objects": [
                    { "id": 1, "x": 16.0, "y": 16.0, "shape": "rect", "width": 8.0, "height": 8.0 }
                ]
            }
        ]
    }"#,
    )
    .unwrap();
    let err = map.validate().unwrap_err();
    assert!(
        matches!(err, ValidationError::DuplicateObjectId { id: 1 }),
        "unexpected error: {err}"
    );
}
