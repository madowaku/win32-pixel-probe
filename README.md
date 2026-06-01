# win32-pixel-probe

win32-pixel-probe is an experimental no-crate Rust Win32 pixel window for tiny games.

It was born from MadoCore 144, but is kept separate so unsafe Win32 FFI experiments do not disturb the stable terminal game core.

Japanese note:

```text
win32-pixel-probe は、MadoCore 144から派生した
外部crateなしWin32ピクセル窓の実験場です。
安定した端末版コアとは分けて、FFIや描画まわりを自由に試します。
```

This repo is the "romance lab" side of the split:

```text
madowaku/madocore144
  Stable tiny game core, Stage Pack, Rule Hooks, FIRST WINDOW terminal build.

madowaku/win32-pixel-probe
  Experiment space for no-crate Win32 FFI, StretchDIBits, PixelLayer display, and input checks.
```

## Probe Result

The first probe is a success:

```text
Win32 Pixel Probe
112,640 bytes
7.64%
Remaining: 1,361,920 bytes
```

That means a native Win32 pixel window is comfortably inside the 1.44 MB target. This project can now grow as a small Win32 game template without forcing `unsafe` FFI into the stable MadoCore terminal repo.

## v0.2 Input & Game Loop Probe

v0.2 adds the first tiny game-loop layer on top of the v0.1.0 window baseline.

- The Win32 timer fires every 16 ms, giving a simple 60Hz-ish tick.
- Held input state is tracked for arrow keys and `W`, `A`, `S`, `D`.
- A small 16x16 sprite moves one pixel per tick and clamps at the 160x144 layer edges.
- The window title reports `win32-pixel-probe v0.2`, layer size, tick style, and the current tick count.
- Rendering still uses the same indexed `PixelLayer` to 32-bit BGRA to `StretchDIBits` path.

## MadoCore Origin

MadoCore 144 is a tiny Rust game core for 1.44 MB game contests. It is not a Unity, Godot, Defold, or GDevelop replacement. It is a small starting point for single-file executable games that keep stages, pixel data, and sound data in code.

FIRST WINDOW is the first complete puzzle game built with MadoCore 144.

Concept: inside a tiny 1.44 MB window, read keys, ice, and traps, then reach the goal.

- 20 one-screen stages.
- Rules use only the existing `classic`, `keydoor`, `ice`, and `trap` hooks.
- Stage order: classic introduction, key-door puzzles, ice puzzles, trap puzzles, and a short finale.
- No external crates, images, sound files, or data files.

## Features

- Windows-friendly terminal demo game.
- No external crates.
- No image, sound, or external data files.
- Text screen buffer with clear, frame, and grid drawing.
- Built-in input commands: `WASD`, arrow-key escape sequences when available, Enter, Esc, `R`, `H`, `U`, `Q`.
- Scenes: Title, Help, Game, Clear, GameOver.
- Grid game structures: `Tile`, `Pos`, `GameState`, `Stage`, `Direction`.
- FIRST WINDOW includes 20 built-in Stage Pack puzzle stages covering classic, keydoor, ice, and trap rules.
- Undo and reset.
- v0.2 Pixel Layer skeleton: 160x144 indexed buffer, 16-color palette, 8x8 tile draw, 16x16 sprite draw, tilemap draw, compact square-wave/MML-style sound data.
- v0.4 Stage Pack metadata: stage id, name, goal type, turn limit, hint, gimmick label, and per-stage tile meanings.
- v0.5 Rule Hooks: compact `classic`, `keydoor`, `ice`, and `trap` rule sets.
- v0.6 Win32 Pixel Probe: feature-gated no-crate Win32 window path for the 160x144 Pixel Layer.
- Package name and release exe are `win32-pixel-probe`.

## Controls

Run the game and type a command, then press Enter.

- `Enter`: start
- `W`, `A`, `S`, `D`: move
- Arrow keys: move when the terminal sends ANSI arrow sequences
- `U`: undo one move
- `R`: reset current stage, or restart after clear
- `H`: help
- `Q` or Esc: quit

## Build

```powershell
cargo fmt
cargo test
cargo build --release
cargo build --release --features win32_pixel
```

To run the experimental Win32 pixel window:

```powershell
cargo run --release --features win32_pixel
```

Use Esc or the window close button to quit. Arrow keys or `W`, `A`, `S`, `D` move the demo sprite while held.

## Size Check

```powershell
(Get-Item .\target\release\win32-pixel-probe.exe).Length
```

For v0.3 capacity measurements:

```powershell
powershell ./size.ps1
```

Target size: `1,474,560 bytes` or less.

Earlier v0.2 measured size log:

| Step | Release exe size |
| --- | ---: |
| Empty Pixel Layer skeleton (`cargo build --release --no-default-features`) | `124,928 bytes` |
| Tile drawing added (`cargo build --release --no-default-features --features pixel_tile`) | `124,928 bytes` |
| Sound data added / v0.2 default build | `124,928 bytes` |
| Contest limit | `1,474,560 bytes` |

MadoCore 144 v0.3 Capacity Demo size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| core pixel+sound, no capacity assets | `124,928 bytes` | `1,349,632 bytes` | `8.47%` |
| `asset_tiles` | `129,024 bytes` | `1,345,536 bytes` | `8.75%` |
| `asset_sprites` | `129,024 bytes` | `1,345,536 bytes` | `8.75%` |
| `asset_maps` | `128,000 bytes` | `1,346,560 bytes` | `8.68%` |
| `asset_sound` | `125,440 bytes` | `1,349,120 bytes` | `8.51%` |
| all capacity assets default | `136,704 bytes` | `1,337,856 bytes` | `9.27%` |

MadoCore 144 v0.4 Stage Pack size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| core pixel+sound, no capacity assets | `128,512 bytes` | `1,346,048 bytes` | `8.72%` |
| `asset_tiles` | `132,608 bytes` | `1,341,952 bytes` | `8.99%` |
| `asset_sprites` | `132,608 bytes` | `1,341,952 bytes` | `8.99%` |
| `asset_maps` | `131,584 bytes` | `1,342,976 bytes` | `8.92%` |
| `asset_sound` | `129,024 bytes` | `1,345,536 bytes` | `8.75%` |
| v0.4 default all capacity assets | `140,288 bytes` | `1,334,272 bytes` | `9.51%` |

MadoCore 144 v0.5 Rule Hooks size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| core pixel+sound, no capacity assets | `131,072 bytes` | `1,343,488 bytes` | `8.89%` |
| `asset_tiles` | `135,680 bytes` | `1,338,880 bytes` | `9.20%` |
| `asset_sprites` | `135,680 bytes` | `1,338,880 bytes` | `9.20%` |
| `asset_maps` | `134,656 bytes` | `1,339,904 bytes` | `9.13%` |
| `asset_sound` | `132,096 bytes` | `1,342,464 bytes` | `8.96%` |
| v0.5 default all capacity assets | `143,360 bytes` | `1,331,200 bytes` | `9.72%` |

FIRST WINDOW v0.1 size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| core pixel+sound, no capacity assets | `135,680 bytes` | `1,338,880 bytes` | `9.20%` |
| `asset_tiles` | `139,776 bytes` | `1,334,784 bytes` | `9.48%` |
| `asset_sprites` | `139,776 bytes` | `1,334,784 bytes` | `9.48%` |
| `asset_maps` | `138,752 bytes` | `1,335,808 bytes` | `9.41%` |
| `asset_sound` | `136,192 bytes` | `1,338,368 bytes` | `9.24%` |
| FIRST WINDOW default all capacity assets | `147,456 bytes` | `1,327,104 bytes` | `10.00%` |

MadoCore 144 v0.6 Win32 Pixel Probe size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| v0.6 Win32 Pixel Probe (`--features win32_pixel`) | `112,640 bytes` | `1,361,920 bytes` | `7.64%` |

The probe is an experiment, not the FIRST WINDOW game port. It keeps the terminal game as the normal build and switches to a native Win32 window only when the `win32_pixel` feature is enabled. The window path uses Rust standard library plus handwritten FFI to `user32.dll` and `gdi32.dll`, creates a 160x144 indexed `PixelLayer`, converts the 16-color palette to 32-bit BGRA, and presents it with `StretchDIBits` at 4x scale. The demo draws a checker background, a few 8x8 tiles, and a moving sprite.

win32-pixel-probe v0.2 Input & Game Loop Probe size log:

| Case | Release exe size | Remaining | Used |
| --- | ---: | ---: | ---: |
| v0.2 Input & Game Loop Probe (`--features win32_pixel`) | `112,640 bytes` | `1,361,920 bytes` | `7.64%` |

## Stage Pack Format

Stages are Rust string constants. Metadata lines begin with `@`, then a blank line separates metadata from the grid.

```text
@stage 01
@name First Window
@goal reach
@rules classic
@limit 40
@hint The shortest path is not always the best.
@gimmick classic
@tile x wall

#######
#P...#
#.#G.#
#....#
#######
```

Supported metadata:

- `@stage`: short stage id.
- `@name`: display name.
- `@goal reach`: clear when the player reaches `G`.
- `@rules classic|keydoor|ice|trap`: small rule hook set.
- `@limit`: turn limit. The game enters GameOver after the move count exceeds this value.
- `@hint`: short text shown in the game view.
- `@gimmick`: a compact label for the stage's special idea.
- `@tile <char> floor|wall|goal|key|door|ice|trap`: per-stage tile meaning override.

Rule hook behavior:

- `classic`: original reach-the-goal behavior.
- `keydoor`: `Key` sets `has_key`; `Door` blocks movement until the key is held.
- `ice`: entering `Ice` slides the player until a wall or obstacle would be hit.
- `trap`: entering `Trap` sets `trapped` and enters GameOver.

FIRST WINDOW uses the same format for its 20 built-in stages. It does not add new engine rules.

Example key-door stage:

```text
@stage 06
@name Brass Key
@goal reach
@rules keydoor
@limit 36
@hint Take K before pushing through D.
@gimmick keydoor
@tile K key
@tile D door

###########
#P.K.D..G#
#.#####..#
#........#
###########
```

## Contest Constraints

- Final release executable plus required files should stay under `1,474,560 bytes`.
- External crates are not used.
- Image files, sound files, and external data files are not used.
- Stages, tile data, sprite data, and sound data live in Rust source.
- Unused features are grouped so they can be removed easily.

## Game Ideas From This Core

- One-screen Sokoban-like puzzle.
- Tiny dungeon key-and-door game.
- Ice sliding puzzle.
- Turn-limited maze.
- Terminal roguelite prototype with Pixel Layer sprites reused later.

## Good Next Ideas

- Keep `size.ps1` as a development helper; do not include it in shipped contest builds unless allowed.
- Add feature flags later if external build configuration is acceptable.
- Add a small stage compressor using code constants, not external files.
- Replace line-based input with Windows console raw input if the size budget still has room.
