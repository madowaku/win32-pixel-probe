# MadoCore 144 v0.4 Stage Pack Design

## Goal

Add a code-embedded Stage Pack format that makes 1.44 MB contest games easier to author without external files or crates.

## Design

`StageMeta` stores the id, name, goal type, turn limit, hint, and gimmick label. `Stage::parse_pack()` reads `@key value` metadata lines followed by a grid. `@tile <char> wall` and `@tile <char> floor` let each stage change tile meanings locally while keeping `P`, `G`, `#`, and `.` familiar.

## Gameplay

The demo uses five packed stages. The game view shows the stage name, move count, turn limit, hint, and gimmick label. Successful movement increments turns. If the count exceeds the limit, the scene changes to `GameOver`. `undo` and `reset` restore the move counter.

## Verification

Tests cover metadata parsing, per-stage tile meanings, turn limit GameOver behavior, undo/reset turn restoration, five demo stages, and the existing movement, wall, goal, Pixel Layer, and capacity asset checks.
