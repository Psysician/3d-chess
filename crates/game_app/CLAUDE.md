# CLAUDE.md

## Overview

This directory contains the Bevy application crate that turns the chess domain into a local playable 3D shell with menus, save-load flow, rendering, input, and integration tests.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and Bevy-to-workspace dependency wiring | Changing crate dependencies or package metadata |
| `README.md` | M3 shell rationale, invariants, and tradeoffs for routing, persistence, and UI ownership | Understanding why the app keeps coarse routes and modal shell resources around `MatchSession` |
| `src/lib.rs` | Module wiring and public re-exports for shell plugins, match state, and test composition | Locating crate entry points or exposing app-layer APIs |
| `src/main.rs` | Native binary entry point for launching the Bevy app | Changing how the app starts from the CLI or desktop launcher |
| `src/app.rs` | App construction, coarse screen states, startup shell resources, and plugin graph | Modifying app lifecycle, top-level routing, or root resource wiring |
| `src/board_coords.rs` | Canonical square-to-world mapping, inverse board lookup, and ray-plane helpers | Debugging board placement, picking drift, or coordinate conversions |
| `src/match_state.rs` | `MatchSession`, launch intents, snapshot conversion, draw state, and recovery dirtiness | Changing match orchestration, load and resume behavior, or Bevy-to-domain boundaries |
| `src/style.rs` | Shell theme colors, camera defaults, and board sizing constants | Adjusting visual direction, board scale, or camera framing |
| `src/plugins/mod.rs` | Plugin module wiring and public exports for live shell, board, input, feedback, and placeholder seams | Reshaping plugin boundaries or re-exporting app-layer systems |
| `src/plugins/app_shell.rs` | Main menu, pause overlay, result flow, promotion UI, and shell button handling | Changing shell presentation, load and resume UI, or top-level state transitions |
| `src/plugins/app_shell_logic.rs` | Pure shell labels, result copy, recovery policy cycling, and button-routing helpers | Changing menu copy, result messaging, or shell decisions without touching Bevy wiring |
| `src/plugins/board_scene.rs` | Procedural board scene, square components, and highlight-state styling | Modifying board visuals, square identity, or selection/check highlighting |
| `src/plugins/input.rs` | Cursor picking, click-to-select flow, keyboard promotion shortcuts, pause overlay toggles, and quick-save input | Debugging interaction flow, promotion input, overlay guards, or keyboard shell actions |
| `src/plugins/menu.rs` | Modal menu state, recovery banner state, and menu action routing for setup, load list, and settings | Changing shell menu behavior or adding new modal menu surfaces |
| `src/plugins/move_feedback.rs` | In-match HUD, draw-claim CTA, save-load status messaging, and piece pulse feedback | Changing turn and check messaging, draw-claim UX, or shell feedback surfaces |
| `src/plugins/piece_view.rs` | Piece mesh/material caches and GameState-driven piece-entity synchronization | Debugging piece placement, render-sync regressions, or piece visuals |
| `src/plugins/save_load.rs` | `SessionStore` integration, autosave and manual-save requests, startup recovery policy, and persisted shell settings | Changing persistence execution, recovery behavior, or settings persistence wiring |
| `src/plugins/save_load_logic.rs` | Pure persistence copy, recovery-banner visibility, and message-composition helpers | Changing save-load copy or recovery visibility rules without touching repository side effects |
| `src/plugins/scaffold.rs` | Placeholder AI and audio plugin seams left inactive in the shipped local shell | Extending deferred seams without disturbing the live M3 plugins |
| `tests/app_shell_logic.rs` | Direct tests for shell labels, result copy, and save-selection helpers | Verifying extracted app-shell logic or debugging copy-only regressions |
| `tests/binary_target.rs` | Binary smoke check for integration test builds | Investigating missing `game_app` binary artifacts in test runs |
| `tests/board_mapping.rs` | Integration assertions for board coordinate roundtrips and off-board rejection | Verifying the public board-coordinate contract or fixing mapping regressions |
| `tests/local_match_flow.rs` | End-to-end local match flow coverage for start, move sync, claim draw, and result transition | Validating playable-loop orchestration or debugging integration regressions |
| `tests/match_state_flow.rs` | Full-shell integration coverage for load intent, startup resume, and pause overlay state transitions | Checking MatchLoading branches, startup recovery, or coarse route regressions |
| `tests/promotion_flow.rs` | Promotion pending and promotion-resolution coverage through the app shell | Debugging promotion overlays, keyboard promotion shortcuts, or piece sync after promotion |
| `tests/save_load_flow.rs` | End-to-end persistence coverage for manual saves, quick-save behavior, recovery policy, and recovery cache clearing | Debugging save-load regressions or validating repository-backed shell flows |
| `tests/save_load_logic.rs` | Direct tests for persistence error joining, recovery-banner visibility, and feedback copy | Verifying extracted save-load helpers or debugging recovery copy regressions |
