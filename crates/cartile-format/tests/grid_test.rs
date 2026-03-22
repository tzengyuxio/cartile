use cartile_format::types::grid::*;

#[test]
fn square_grid_serde_roundtrip() {
    let grid = Grid {
        grid_type: GridType::Square,
        width: 100,
        height: 80,
        tile_width: 16,
        tile_height: 16,
        orientation: None,
        stagger: None,
        topology: Topology::Bounded,
        projection: Projection {
            projection_type: ProjectionType::Orthogonal,
            angle: None,
            extra: Default::default(),
        },
        height_mode: HeightMode::None,
        extra: Default::default(),
    };
    let json = serde_json::to_string(&grid).unwrap();
    let back: Grid = serde_json::from_str(&json).unwrap();
    assert_eq!(back.grid_type, GridType::Square);
    assert_eq!(back.width, 100);
    assert_eq!(back.topology, Topology::Bounded);
    assert_eq!(back.height_mode, HeightMode::None);
}

#[test]
fn hex_grid_serde_roundtrip() {
    let grid = Grid {
        grid_type: GridType::Hexagonal,
        width: 20,
        height: 20,
        tile_width: 32,
        tile_height: 28,
        orientation: Some(HexOrientation::PointyTop),
        stagger: Some(Stagger {
            axis: StaggerAxis::Y,
            index: StaggerIndex::Odd,
        }),
        topology: Topology::WrapXy,
        projection: Projection {
            projection_type: ProjectionType::Orthogonal,
            angle: None,
            extra: Default::default(),
        },
        height_mode: HeightMode::Stepped,
        extra: Default::default(),
    };
    let json = serde_json::to_string_pretty(&grid).unwrap();
    let back: Grid = serde_json::from_str(&json).unwrap();
    assert_eq!(back.grid_type, GridType::Hexagonal);
    assert_eq!(back.orientation, Some(HexOrientation::PointyTop));
    assert!(back.stagger.is_some());
    assert_eq!(back.topology, Topology::WrapXy);
}

#[test]
fn oblique_projection_with_angle() {
    let json = r#"{"type":"oblique","angle":26.57}"#;
    let proj: Projection = serde_json::from_str(json).unwrap();
    assert_eq!(proj.projection_type, ProjectionType::Oblique);
    assert_eq!(proj.angle, Some(26.57));
}

#[test]
fn topology_default_is_bounded() {
    let json = r#"{
        "type": "square", "width": 10, "height": 10,
        "tile_width": 16, "tile_height": 16,
        "projection": {"type": "orthogonal"}
    }"#;
    let grid: Grid = serde_json::from_str(json).unwrap();
    assert_eq!(grid.topology, Topology::Bounded);
    assert_eq!(grid.height_mode, HeightMode::None);
}
