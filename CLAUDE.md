# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**cartile** — *One format. Every engine.*

A universal tilemap toolkit with a cross-engine runtime and editor, built in Rust. The name blends *cartography* and *tile*.

- **Status**: Phase 1 complete (core library, CLI, WASM bindings)
- **License**: MIT

## Vision

Become the "Spine of tilemap" — a single editor + unified runtime model that works across all 2D game engines, just as Spine did for skeletal animation.

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

Core logic is written once in Rust. Each engine gets a thin binding layer (type conversion, lifecycle management) — **no business logic in bindings**. This ensures 100% consistent behavior across all engines.

## Tech Stack

- **Core**: Rust
- **WASM**: wasm-bindgen, wasm-pack
- **Format**: JSON with JSON Schema validation
- **Editor (Phase 2)**: Rust + WASM + WebGPU (fallback to Canvas 2D)
- **Engine bindings**: Godot (gdext), Unity (C# P/Invoke), Bevy (Rust crate), PixiJS (wasm-bindgen)

## Workspace Structure

This is a Cargo workspace with three crates:

```
crates/
├── cartile-format/   # Core library: types, parsing, validation, auto-tiling
├── cartile-cli/      # CLI tool: convert, export, validate
└── cartile-wasm/     # WASM bindings for browser/JS usage
```

## Build Commands

```bash
cargo build                            # build all crates
cargo test                             # run all tests
cargo test -p cartile-format           # test a single crate
cargo fmt                              # format code
cargo clippy                           # lint
wasm-pack build crates/cartile-wasm --target web   # build WASM target
cargo install --path crates/cartile-cli             # install CLI locally
```

## Development Phases

### Phase 1 — Foundation (complete)
- Format spec v0.1 (JSON Schema): orthogonal/isometric/hex tilemap, tileset definitions, layers, custom properties
- Rust core library: format read/write/validate, bitmask-based auto-tiling, tileset management
- CLI tool: format conversion (TMX/LDtk JSON ↔ cartile format)
- Runtime bindings: Godot GDExtension + JS/WASM

### Phase 2 — Editor MVP
- Web-based editor (Rust + WASM + WebGPU)
- Tilemap painting, layer management, auto-tiling, import from Tiled/LDtk
- Additional bindings: Bevy, Unity

### Phase 3 — Differentiation
- Smart auto-tiling (rule inference), animated tiles, entity/trigger/collision editing
- Procedural generation, native desktop app (wgpu)

## Format Design Principles

1. **Human-readable** — JSON, hand-editable
2. **Schema-validated** — JSON Schema for editor and CI
3. **VCS-friendly** — Stable structure, diff-friendly
4. **Forward-compatible** — Versioned schema, unknown fields preserved
5. **Streaming-parseable** — Large maps loadable in chunks
6. **Engine-agnostic** — No assumptions about coordinate systems or render pipelines
7. **Composable** — Tilesets separated from maps, reusable across maps

## Key Design Decisions

- **Rust + WASM**: Single codebase → native lib (C ABI), WASM module, CLI, desktop app. Best WASM toolchain + memory safety.
- **Not Electron**: LDtk/Ogmo use Haxe+Electron (~200MB, limited). WASM is lighter and embeddable.
- **Spine model**: Core + thin binding per engine, not separate implementations (Tiled's weakness).

## Code Style

- Code comments and variable names in **English**
- Commit messages follow **Conventional Commits**: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Follow Rust community conventions (rustfmt, clippy)
- Run `cargo fmt` and `cargo clippy` before committing

## Related Documents

- `docs/usage-guide.md` — CLI reference and library usage guide
- `docs/specs/feasibility-analysis.md` — Full market feasibility analysis and product design
- `docs/specs/format-spec-v0.1-design.md` — Format spec v0.1 design document
- `docs/research/tilemap-tools-landscape.md` — Competitive landscape research
