# MadoCore 144 Notes

## Purpose

MadoCore 144 is a compact game core for making Rust games that fit under a 1.44 MB contest limit. The first target is a Windows terminal executable with no external crates and no external assets.

## v0.1 Terminal Core

The v0.1 layer keeps the game simple:

- `Screen` owns a text buffer and renders it to standard output.
- `Stage` parses code-embedded string maps.
- `GameState` owns scene, stage index, player position, move count, and undo history.
- `Direction`, `Tile`, and `Pos` are plain data types.

Stages use this format:

```text
#######
#P...G#
#..#..#
#######
```

`P` is the player start, `G` is the goal, `#` is a wall, and `.` is floor.

## v0.2 Pixel Layer

The Pixel Layer is intentionally independent from the terminal game logic. It currently provides:

- Indexed low-resolution pixel buffer.
- 16-color palette.
- 8x8 tile drawing.
- 16x16 sprite drawing with a transparent color.
- Tilemap drawing.
- Terminal preview conversion.
- Compact square-wave sound effect data and MML-style BGM text.

The terminal demo does not depend on real image or audio files. Future entrants can replace the preview with a Windows-specific framebuffer or keep the terminal version.

## v0.3 Capacity Demo

The Capacity Demo adds feature-gated dummy assets that are referenced by `main` through `capacity_demo_checksum()`, so release builds measure data that is actually reachable:

- `asset_tiles`: 64 generated 8x8 indexed tiles.
- `asset_sprites`: 16 generated 16x16 indexed sprites.
- `asset_maps`: 10 generated 20x15 tilemaps.
- `asset_sound`: 3 MML-style BGM strings and 8 square-wave SE patterns.

Use `powershell ./size.ps1` to build and measure the base core, each asset group, and the default all-assets build.

## Size Strategy

The release profile uses:

- `opt-level = "z"`
- `lto = true`
- `codegen-units = 1`
- `panic = "abort"`
- `strip = true`

The code avoids unused abstractions, generic engines, external crates, and file loaders. Pixel Layer and sound code are grouped so they can be deleted when a specific contest entry does not need them.

## Extension Ideas

- Add a tiny tile/sprite editor encoded as Rust constants.
- Add a simple stage-pack format stored as string constants.
- Add a small animation table for player direction.
- Add a Win32 output path only if no-crate FFI remains below the size limit.
- Add optional raw input after checking release size impact.
