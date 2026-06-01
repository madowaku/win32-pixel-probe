# MadoCore 144 Notes

## Repository Split

This repository is now `win32-pixel-probe`, a separate experiment space born from MadoCore 144. The split keeps the project roles clear:

- `madowaku/madocore144`: stable 1.44 MB game core, Stage Pack, Rule Hooks, and FIRST WINDOW terminal build.
- `madowaku/win32-pixel-probe`: no-crate Win32 FFI, `StretchDIBits`, `PixelLayer` display, and input experiments.

The probe can be treated as a tiny Win32 game template if it keeps its size and code shape. Later, stable pieces can be reverse-imported into MadoCore as a feature, spun into a `madocore144-pixel` branch, or kept here as an independent tool.

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

## v0.4 Stage Pack

The Stage Pack turns code-embedded stage strings into a compact authoring format. A stage can define:

- `@stage`: id.
- `@name`: display name.
- `@goal reach`: goal type.
- `@limit`: turn limit.
- `@hint`: player-facing hint.
- `@gimmick`: a short design label.
- `@tile <char> wall` or `@tile <char> floor`: per-stage tile meanings.

The parser still keeps the old `Stage::parse(&[...])` helper for tiny tests and experiments. The demo uses `Stage::parse_pack()` and includes nine stages after the v0.5 Rule Hooks examples.

Turns are counted only for successful movement. Wall bumps do not consume turns. If movement makes the counter greater than the stage limit, the scene becomes `GameOver`. `undo` and `reset` restore the move counter and scene state.

## v0.5 Rule Hooks

Rule Hooks add a small `RuleSet` field to `StageMeta`:

- `classic`: the original reach-the-goal rule.
- `keydoor`: `Key` sets `has_key`; `Door` requires `has_key`.
- `ice`: entering `Ice` slides until the next step would hit a wall or blocked tile.
- `trap`: entering `Trap` sets `trapped` and changes the scene to `GameOver`.

The tile enum now represents simple effects: `Floor`, `Wall`, `Goal`, `Key`, `Door`, `Ice`, and `Trap`. Stage packs can map characters with `@tile <char> floor|wall|goal|key|door|ice|trap`.

Movement is split into small match-based hooks:

- `can_enter_tile()`
- `on_enter_tile()`
- `after_move()`
- `check_clear()`
- `check_game_over()`

This keeps game-specific behavior replaceable without a trait hierarchy or a large engine abstraction. `Snapshot` stores the rule flags so `undo` and `reset` restore `has_key` and `trapped`.

## FIRST WINDOW v0.1

FIRST WINDOW is the first game built with MadoCore 144. It uses the v0.5 Rule Hooks and Stage Pack format as-is: no new engine rules, no external assets, and no external data files.

The game contains 20 one-screen stages:

- 01-04: classic introduction.
- 05-08: key-door puzzles.
- 09-13: ice puzzles.
- 14-17: trap puzzles.
- 18-20: finale stages that reuse the same single-rule hook model.

## v0.6 Win32 Pixel Probe

The Win32 Pixel Probe is an experiment branch for proving that the existing `PixelLayer` can reach a native Windows window without external crates. It is not a FIRST WINDOW port yet.

Build selection:

- Normal builds keep the terminal game loop.
- `--features win32_pixel` changes `main()` to a Windows pixel-window demo.
- The feature depends on `pixel_tile` so the existing tilemap and sprite drawing helpers remain the shared drawing surface.

Rendering path:

- A 160x144 `PixelLayer` stores 4-bit palette indices in a `u8` buffer.
- The demo frame draws a checker background, a small 8x8 tilemap, and one 16x16 sprite.
- The palette converter expands each index to a `u32` BGRA word.
- The Win32 paint handler calls `StretchDIBits` with a top-down 32-bit DIB and scales the layer to 4x.

Win32 boundary:

- The handwritten FFI lives in a small `win32_pixel` module in `src/main.rs`.
- It calls `RegisterClassW`, `CreateWindowExW`, `GetMessageW`, `DispatchMessageW`, `BeginPaint`, `EndPaint`, and `StretchDIBits`.
- Esc and the window close button exit. Arrow key state is tracked and nudges the demo sprite.

Risk notes:

- `unsafe` is isolated to the FFI module and one single-window state pointer.
- The terminal game remains the stable path if the window experiment becomes too complex.
- The next decision should be based on release size, code readability, and whether FIRST WINDOW can reuse this path without turning the core into a Win32 framework.

## win32-pixel-probe v0.2 Input & Game Loop Probe

The separate probe repo now owns the first game-loop experiment:

- A Win32 timer runs at 16 ms per event, which is treated as a simple 60Hz-ish tick source.
- `ProbeInput` stores held direction flags. Win32 key messages map both arrow keys and `WASD` into those flags.
- `ProbeGame` is safe Rust state outside the FFI module. It owns the sprite position and tick count.
- Each tick moves the sprite one pixel per axis and clamps it inside the 160x144 layer.
- The window title is refreshed from safe state and shows the version, layer size, tick style, and tick count.

This keeps the Win32 module focused on message handling, title updates, and `StretchDIBits`, while movement rules stay testable without opening a window.

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
