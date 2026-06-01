# MadoCore 144 Design

## Goal

Build a tiny Rust game core for 1.44 MB game contests. It runs as a single Windows-friendly terminal executable, uses no external crates, and keeps assets and stages in code.

## Scope

Version 0.1 provides a terminal grid game core: screen buffer drawing, simple input, scene switching, stage parsing, movement, undo, reset, and a three-stage demo puzzle.

Version 0.2 adds a small Pixel Layer module: low-resolution indexed pixels, a palette, 8x8 tile drawing, 16x16 sprite drawing, tilemap drawing, animation ticks, and compact square-wave/MML-style data structures. The terminal demo still stays text-first so the release executable remains small and portable.

## Architecture

The first implementation is intentionally one Rust source file. `Screen` handles terminal text drawing. `Stage` owns parsed static stage data. `GameState` owns player state, history, and current stage. `PixelLayer` is isolated so contest entrants can delete it if they only need the v0.1 terminal core.

## Input

The core supports `WASD`, lowercase equivalents, `R`, `H`, `U`, `Q`, Enter, Esc, and ANSI arrow-key sequences when the terminal provides them. Input is read from standard input without raw-mode dependencies.

## Testing

Unit tests cover stage parsing, wall collision, goal detection, undo, reset, Pixel Layer tile drawing, and sprite drawing. Manual runtime checks use `cargo run`.

## Size Policy

Release builds use size-oriented profile settings. The README records measured sizes for the empty Pixel Layer, tile drawing, and sound-data additions. These measurements are taken from the final release artifact where possible.
