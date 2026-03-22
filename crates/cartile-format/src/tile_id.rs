use serde::{Deserialize, Serialize};

const FLIP_HORIZONTAL: u32 = 0x80000000;
const FLIP_VERTICAL: u32 = 0x40000000;
const FLIP_DIAGONAL: u32 = 0x20000000;
const FLAG_MASK: u32 = 0xE0000000;
const GID_MASK: u32 = 0x1FFFFFFF;

/// A tile identifier that encodes both the global tile ID and transform flags
/// in a single u32. Upper 3 bits are flip flags, lower 29 bits are the GID.
///
/// Bit layout (matches Tiled's encoding):
/// - bit 31: flip horizontal
/// - bit 30: flip vertical
/// - bit 29: flip diagonal (anti-diagonal)
/// - bits 28-0: global tile ID (GID), 0 = empty
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TileId(u32);

impl TileId {
    /// An empty tile (GID 0, no flags).
    pub const EMPTY: TileId = TileId(0);

    /// Construct a `TileId` directly from a raw u32 (GID + flags combined).
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Construct a `TileId` from a plain GID with no flip flags.
    pub fn from_gid(gid: u32) -> Self {
        debug_assert!(gid <= GID_MASK, "GID exceeds 29-bit maximum");
        Self(gid & GID_MASK)
    }

    /// Construct a `TileId` from a GID and explicit flip flags.
    pub fn new(gid: u32, flip_h: bool, flip_v: bool, flip_d: bool) -> Self {
        debug_assert!(gid <= GID_MASK, "GID exceeds 29-bit maximum");
        let mut raw = gid & GID_MASK;
        if flip_h {
            raw |= FLIP_HORIZONTAL;
        }
        if flip_v {
            raw |= FLIP_VERTICAL;
        }
        if flip_d {
            raw |= FLIP_DIAGONAL;
        }
        Self(raw)
    }

    /// Returns the raw u32 representation (GID + flags).
    pub fn raw(self) -> u32 {
        self.0
    }

    /// Returns the global tile ID (lower 29 bits).
    pub fn gid(self) -> u32 {
        self.0 & GID_MASK
    }

    /// Returns `true` if this is an empty tile (raw value == 0).
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if the horizontal flip flag is set.
    pub fn flip_horizontal(self) -> bool {
        self.0 & FLIP_HORIZONTAL != 0
    }

    /// Returns `true` if the vertical flip flag is set.
    pub fn flip_vertical(self) -> bool {
        self.0 & FLIP_VERTICAL != 0
    }

    /// Returns `true` if the diagonal (anti-diagonal) flip flag is set.
    pub fn flip_diagonal(self) -> bool {
        self.0 & FLIP_DIAGONAL != 0
    }

    /// Returns only the flag bits (upper 3 bits).
    pub fn flags(self) -> u32 {
        self.0 & FLAG_MASK
    }
}
