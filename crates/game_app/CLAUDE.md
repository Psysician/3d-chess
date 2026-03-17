# CLAUDE.md

## Overview

This directory contains the Bevy application crate that turns the chess domain into a local playable 3D shell with menus, save-load flow, rendering, input, and integration tests.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest, feature flags for `automation-transport`, and Bevy-to-workspace dependency wiring | Changing crate dependencies, adding optional features, or package metadata |
| `README.md` | M3 shell rationale, invariants, and tradeoffs for routing, persistence, and UI ownership | Understanding why the app keeps coarse routes and modal shell resources around `MatchSession` |
| `src/lib.rs` | Module wiring and public re-exports for shell plugins, match state, automation contract, and test composition | Locating crate entry points, exposing automation types, or adding app-layer APIs |
| `src/main.rs` | Native binary entry point for launching the Bevy app | Changing how the app starts from the CLI or desktop launcher |
| `src/bin/game_app_agent.rs` | Feature-gated agent binary that boots the automation harness and runs the stdio JSON Lines session | Changing the agent startup sequence or stdio entry point |
| `src/app.rs` | App construction, coarse screen states, startup shell resources, plugin graph, and headless app builder | Modifying app lifecycle, top-level routing, headless harness wiring, or root resource setup |
| `src/automation.rs` | Semantic automation command and snapshot types, `AutomationHarness`, and snapshot capture from shell resources | Adding automation commands, changing snapshot fields, or building a harness |
| `src/automation_transport.rs` | JSON Lines stdio transport framing `AutomationCommand` and `AutomationSnapshot` over request/response envelopes | Adding transport adapters, changing serialization, or debugging agent I/O |
| `src/board_coords.rs` | Canonical square-to-world mapping, inverse board lookup, and ray-plane helpers | Debugging board placement, picking drift, or coordinate conversions |
| `src/match_state.rs` | `MatchSession`, launch intents, snapshot conversion, draw state, recovery dirtiness, and stable player-visible snapshot accessors | Changing match orchestration, load and resume behavior, snapshot accessors, or Bevy-to-domain boundaries |
| `src/style.rs` | Shell theme colors, camera defaults, and board sizing constants | Adjusting visual direction, board scale, or camera framing |
| `src/plugins/mod.rs` | Plugin module wiring and public exports for live shell, board, input, feedback, automation, and placeholder seams | Reshaping plugin boundaries or re-exporting app-layer systems |
| `src/plugins/app_shell.rs` | Main menu, pause overlay, result flow, promotion UI, shared navigation/save/settings/confirmation handlers, and shell button dispatch | Changing shell presentation, load and resume UI, extracting semantic handlers, or top-level state transitions |
| `src/plugins/automation.rs` | `AutomationPlugin` command queue dispatch, `AutomationSnapshotResource`, and `AutomationHarness::try_submit` | Adding new command variants, changing dispatch order, or wiring semantic automation into the plugin graph |
| `src/plugins/app_shell_logic.rs` | Pure shell labels, result copy, recovery policy cycling, and button-routing helpers | Changing menu copy, result messaging, or shell decisions without touching Bevy wiring |
| `src/plugins/board_scene.rs` | Procedural board scene, square components, and highlight-state styling | Modifying board visuals, square identity, or selection/check highlighting |
| `src/plugins/input.rs` | Cursor picking, shared square interaction and promotion helpers, keyboard shortcuts, pause overlay toggles, and quick-save input | Debugging interaction flow, adding shared match action helpers, or fixing promotion or overlay guards |
| `src/plugins/menu.rs` | Modal menu state, recovery banner state, and menu action routing for setup, load list, and settings | Changing shell menu behavior or adding new modal menu surfaces |
| `src/plugins/move_feedback.rs` | In-match HUD, draw-claim CTA, save-load status messaging, and piece pulse feedback | Changing turn and check messaging, draw-claim UX, or shell feedback surfaces |
| `src/plugins/piece_view.rs` | Piece mesh/material caches and GameState-driven piece-entity synchronization | Debugging piece placement, render-sync regressions, or piece visuals |
| `src/plugins/save_load.rs` | `SessionStore` integration, semantic save/load/settings requests, startup recovery policy, and persisted shell settings | Changing persistence execution, adding new save requests, recovery behavior, or settings persistence wiring |
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
| `tests/automation_harness.rs` | Integration coverage asserting the headless harness boots to `MainMenu` and that `build_app` preserves the default startup contract | Verifying harness boot behavior or checking headless-vs-windowed parity |
| `tests/automation_semantic_flow.rs` | End-to-end automation coverage for start, move, save, load, promotion, recovery, rematch, and return-to-menu flows via `try_submit` | Adding semantic command coverage or debugging automation routing regressions |
| `tests/automation_transport.rs` | Feature-gated stdio transport contract coverage for round-trip serialization and structured error responses | Debugging transport framing, adding new command serialization tests, or checking error codes |
