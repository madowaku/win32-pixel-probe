# MadoCore 144 v0.5 Rule Hooks Design

## Goal

Let Stage Pack entries choose a tiny rule set so different games can reuse the core without traits, external crates, or large abstractions.

## Rule Sets

`StageMeta.rule_set` supports `classic`, `keydoor`, `ice`, and `trap`. Tile effects are represented directly in the small `Tile` enum: `Floor`, `Wall`, `Goal`, `Key`, `Door`, `Ice`, and `Trap`.

## Movement Hooks

`GameState::move_player()` delegates to compact helper functions: `can_enter_tile()`, `on_enter_tile()`, `after_move()`, `check_clear()`, and `check_game_over()`. The implementation stays match-based. `has_key` and `trapped` live on `GameState`, and snapshots store those flags for undo.

## Demo

The demo now includes nine packed stages, including examples for classic, key-door, ice sliding, and trap behavior.

## Verification

Tests cover rule parsing, tile effect parsing, key pickup, door blocking, door entry with key, ice sliding, trap GameOver, classic compatibility, and undo/reset flag restoration.
