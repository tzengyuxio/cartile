# CLAUDE.md — cartile

## Project Overview

**cartile** — *One format. Every engine.*

A universal tilemap toolkit with a cross-engine runtime and editor, built in Rust. The name blends *cartography* and *tile* — because your maps should work everywhere you take them.

- **GitHub description**: A universal tilemap toolkit — editor, format, and runtime for every engine.
- **License**: MIT (planned)
- **Language**: Rust (core library, CLI, editor via WASM)
- **Status**: Pre-development (Phase 1 planning)

## Vision

Become the "Spine of tilemap" — just as Spine became the de facto standard for 2D skeletal animation through its editor + unified runtime model, cartile aims to do the same for 2D tilemaps.

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │         Rust Core Library            │
                    │   (format, auto-tiling, tileset)     │
                    └──────┬──────────────┬────────────────┘
                           │              │
              ┌────────────▼───┐    ┌─────▼──────────┐
              │   C ABI (.so)  │    │  WASM (.wasm)  │
              │   Native path  │    │   Web path     │
              └──┬────┬────┬───┘    └──┬─────────┬───┘
                 │    │    │           │         │
                 ▼    ▼    ▼           ▼         ▼
              Godot Unity Bevy    Web Editor  PixiJS
```

Core logic is written once in Rust. Each engine gets a thin binding layer (type conversion, lifecycle management) — no business logic in bindings. This ensures 100% consistent behavior across all engines.

## Tech Stack

- **Core**: Rust
- **WASM**: wasm-bindgen, wasm-pack
- **Format**: JSON with JSON Schema validation
- **Editor (Phase 2)**: Rust + WASM + WebGPU (fallback to Canvas 2D)
- **Native desktop (Phase 3)**: Same Rust codebase via wgpu
- **Engine bindings**:
  - Godot: GDExtension via `gdext` crate
  - Unity: Native Plugin + C# P/Invoke wrapper
  - Bevy: Direct Rust crate dependency
  - PixiJS/Web: WASM + wasm-bindgen auto-generated JS/TS wrapper

## Development Phases

### Phase 1 — Foundation (current target, ~4-6 months)
- Format spec v0.1 (JSON Schema)
  - Scope: orthogonal/isometric/hex tilemap, tileset definitions, tile layer + object layer, custom properties
  - NOT included: entity system, procedural generation (Phase 3)
- Rust core library: format read/write/validate, bitmask-based auto-tiling, tileset management
- CLI tool: format conversion (TMX/LDtk JSON ↔ cartile format, bidirectional)
- Runtime bindings: Godot GDExtension + JS/WASM
- Runtime also supports direct TMX reading (zero migration cost for Tiled users)

### Phase 2 — Editor MVP (~6-9 months)
- Web-based editor (Rust + WASM + WebGPU, fallback Canvas 2D)
- Core features: tilemap painting, layer management, basic auto-tiling
- Import existing Tiled/LDtk projects
- Additional bindings: Bevy, Unity

### Phase 3 — Differentiation (ongoing)
- Smart auto-tiling (one-click rule inference)
- Animated tile editing + live preview
- Entity / trigger / collision editing
- Procedural generation rules integration
- Native desktop app (same Rust codebase via wgpu)

## Format Design Principles

1. **Human-readable** — JSON, developers can read and hand-edit
2. **Schema-validated** — Full JSON Schema for editor and CI validation
3. **Version-control friendly** — Stable structure, diff-friendly, no unnecessary field reordering
4. **Forward-compatible** — Versioned schema, new versions read old formats, unknown fields preserved
5. **Streaming-parseable** — Large maps can be loaded in chunks
6. **Engine-agnostic** — No assumptions about coordinate systems, render pipelines, or physics engines
7. **Composable** — Tileset definitions separated from map definitions, same tileset reusable across maps

## Code Style

- Code comments and variable names in **English**
- Commit messages follow **Conventional Commits**: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Follow Rust community conventions (rustfmt, clippy)
- Run `cargo fmt` and `cargo clippy` before committing

## Key Design Decisions

- **Why Rust + WASM**: Single codebase deploys as native library (C ABI/FFI), WASM module (web editor + web games), CLI tool, and desktop app. Rust has the most mature WASM toolchain, memory safety for reliable runtime embedding, and an active game dev community.
- **Why not Electron**: LDtk and Ogmo both use Haxe + Electron — bloated (~200MB), limited ecosystem. WASM is lighter and embeddable.
- **Why not C++**: Higher contribution barrier, complex WASM toolchain (Emscripten), manual memory management.
- **Spine architecture model**: Core + thin binding per engine, not separate implementations per engine (which is Tiled's weakness).

## Related Documents

- `docs/specs/feasibility-analysis.md` — Full market feasibility analysis and product design
- `docs/research/tilemap-tools-landscape.md` — Competitive landscape research
