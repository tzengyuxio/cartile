use cartile_format::tile_id::TileId;

#[test]
fn empty_tile() {
    let tid = TileId::EMPTY;
    assert_eq!(tid.raw(), 0);
    assert!(tid.is_empty());
    assert_eq!(tid.gid(), 0);
    assert!(!tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn simple_gid() {
    let tid = TileId::from_gid(42);
    assert_eq!(tid.gid(), 42);
    assert!(!tid.is_empty());
    assert!(!tid.flip_horizontal());
}

#[test]
fn horizontal_flip() {
    let tid = TileId::from_raw(0x80000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn rotation_90_cw() {
    let tid = TileId::from_raw(0xA0000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(tid.flip_diagonal());
}

#[test]
fn rotation_180() {
    let tid = TileId::from_raw(0xC0000001);
    assert_eq!(tid.gid(), 1);
    assert!(tid.flip_horizontal());
    assert!(tid.flip_vertical());
    assert!(!tid.flip_diagonal());
}

#[test]
fn max_gid() {
    let tid = TileId::from_gid(0x1FFFFFFF);
    assert_eq!(tid.gid(), 0x1FFFFFFF);
    assert!(!tid.flip_horizontal());
}

#[test]
fn builder_flags() {
    let tid = TileId::new(5, true, false, true);
    assert_eq!(tid.gid(), 5);
    assert!(tid.flip_horizontal());
    assert!(!tid.flip_vertical());
    assert!(tid.flip_diagonal());
    assert_eq!(tid.raw(), 0xA0000005);
}

#[test]
fn serde_roundtrip() {
    let tid = TileId::from_raw(0x80000001);
    let json = serde_json::to_string(&tid).unwrap();
    assert_eq!(json, "2147483649");
    let back: TileId = serde_json::from_str(&json).unwrap();
    assert_eq!(back.raw(), tid.raw());
}
