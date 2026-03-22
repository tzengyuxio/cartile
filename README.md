# cartile

*One format. Every engine.*

A universal tilemap toolkit with a cross-engine runtime and editor. The name blends *cartography* and *tile* — because your maps should work everywhere you take them.

## What is cartile?

cartile is an open-source tilemap system that provides:

- **A unified tilemap format** — A well-defined, JSON-based format with full schema validation, designed for cross-engine portability and version control friendliness
- **A cross-engine runtime** — A single Rust core library with official bindings for Godot, Unity, Bevy, PixiJS, and more. Same behavior, every engine.
- **A modern editor** — A web-native tilemap editor built with Rust + WASM, featuring smart auto-tiling, animated tiles, and live preview (coming in Phase 2)

## Why?

Existing tilemap tools have gaps:

| Problem | How cartile solves it |
|---------|----------------------|
| **Tiled's runtimes are community-maintained** — behavior differs across engines | One Rust core, thin binding per engine. 100% consistent. |
| **No standard tilemap format** — TMX is XML-era, LDtk JSON is editor-internal | Purpose-built format with JSON Schema, designed for interop |
| **Editors use legacy tech** — C++/Qt (Tiled) or Haxe/Electron (LDtk) | Rust + WASM: native performance, zero-install web access |
| **Auto-tiling is painful** — 47-tile blob rules are tedious to set up | Smart auto-tiling with rule inference (Phase 3) |

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

## Status

**Pre-development** — Currently in Phase 1 planning. See [Feasibility Analysis](docs/specs/feasibility-analysis.md) for the full product design.

## License

MIT
