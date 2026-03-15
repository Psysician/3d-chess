# CLAUDE.md

## Overview

This directory contains the Bevy application crate that turns the chess domain into a local playable 3D shell with rendering, UI, input, and integration tests.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and Bevy-to-workspace dependency wiring | Changing crate dependencies or package metadata |
| `README.md` | M2 architectural rationale, invariants, and scope boundaries for the app shell | Understanding why `game_app` is structured around `MatchSession`, board coords, and narrow M2 scope |
| `src/lib.rs` | Module wiring and public re-exports for test and app composition | Locating crate entry points or exposing app-layer APIs |
| `src/main.rs` | Native binary entry point for launching the Bevy app | Changing how the app starts from the CLI or desktop launcher |
| `src/app.rs` | App construction, global state enum, plugin graph, and window setup | Modifying app lifecycle, screen states, or top-level plugin registration |
| `src/board_coords.rs` | Canonical square-to-world mapping, inverse board lookup, and ray-plane helpers | Debugging board placement, picking drift, or coordinate conversions |
| `src/match_state.rs` | `MatchSession`, claim-draw state, last-move tracking, and chess-core bridge helpers | Changing match orchestration, draw handling, or Bevy-to-domain boundaries |
| `src/style.rs` | Shell theme colors, camera defaults, and board sizing constants | Adjusting visual direction, board scale, or camera framing |
| `src/plugins/mod.rs` | Plugin module wiring and public plugin/component exports | Reshaping plugin boundaries or re-exporting app-layer systems |
| `src/plugins/app_shell.rs` | Main menu, loading/result flow, promotion overlay, and shared shell buttons | Changing screen transitions, menu/result UI, or promotion shell affordances |
| `src/plugins/board_scene.rs` | Procedural board scene, square components, and highlight-state styling | Modifying board visuals, square identity, or selection/check highlighting |
| `src/plugins/input.rs` | Cursor picking, click-to-select flow, keyboard promotion shortcuts, and in-match escape behavior | Debugging interaction flow, promotion input, or square selection rules |
| `src/plugins/move_feedback.rs` | In-match HUD, draw-claim CTA, and piece pulse feedback | Changing turn/check messaging, draw-claim UX, or move emphasis behavior |
| `src/plugins/piece_view.rs` | Piece mesh/material caches and GameState-driven piece-entity synchronization | Debugging piece placement, render-sync regressions, or piece visuals |
| `src/plugins/scaffold.rs` | Placeholder plugins reserved for later milestones such as save/load, AI, audio, and menu expansion | Extending deferred seams without disturbing the live M2 plugins |
| `tests/binary_target.rs` | Binary smoke check for integration test builds | Investigating missing `game_app` binary artifacts in test runs |
| `tests/board_mapping.rs` | Integration assertions for board coordinate roundtrips and off-board rejection | Verifying the public board-coordinate contract or fixing mapping regressions |
| `tests/local_match_flow.rs` | End-to-end local match flow coverage for start, move sync, claim draw, and result transition | Validating playable-loop orchestration or debugging integration regressions |
| `tests/match_state_flow.rs` | Shell state-path smoke tests for start, rematch, and menu return behavior | Checking screen-state regressions or `MatchSession` reset behavior |
| `tests/promotion_flow.rs` | Promotion pending and promotion-resolution coverage through the app shell | Debugging promotion overlays, keyboard promotion shortcuts, or piece sync after promotion |
