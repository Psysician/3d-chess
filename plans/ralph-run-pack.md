# Ralph Run Pack

Use this as the operator-facing summary. The canonical execution files are:

- `plans/ralph-prompt.md`
- `plans/ralph-task-list.md`
- `plans/ralph-verification.md`

## Objective

Give Ralph a multi-hour, repo-grounded execution pack that pushes `3d-chess` from "functional M3 local shell" toward "more polished local-play build" while moving honest automated coverage to a hard, reproducible 90% path.

## Verified Baseline

- `cargo test --workspace` passes locally.
- The repo is a Rust workspace with four crates:
  - `crates/chess_core`
  - `crates/chess_persistence`
  - `crates/engine_uci`
  - `crates/game_app`
- `README.md` still describes the shipped scope as the M3 local playable shell.
- `chess_core` remains the rules authority.
- `MatchSession` remains the Bevy-facing gameplay bridge.
- `AiMatchPlugin` and `ChessAudioPlugin` are still empty scaffolds; do not treat them as live systems.

Fresh verified coverage run:

- Command:
  - `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-run`
- Result:
  - pass
- Generated artifacts:
  - `/tmp/3d-chess-coverage-run/report.json`
  - `/tmp/3d-chess-coverage-run/workspace.lcov`
  - `/tmp/3d-chess-coverage-run/coverage-summary.json`
  - `/tmp/3d-chess-coverage-run/summary.txt`
  - `/tmp/3d-chess-coverage-run/thresholds.env`

Current honest baseline from generated artifacts:

- workspace: `90.09%`
- chess_core: `89.95%`
- chess_persistence: `95.49%`
- engine_uci: `100.00%`
- game_app: `89.29%`

Measured hotspot files:

- `crates/game_app/src/app.rs` at `66.67%`
- `crates/game_app/src/board_coords.rs` at `69.23%`
- `crates/game_app/src/plugins/app_shell_logic.rs` at `83.19%`
- `crates/game_app/src/plugins/app_shell.rs` at `84.59%`
- `crates/chess_core/src/game.rs` at `86.47%`

Important caveat:

- Direct `cargo llvm-cov --workspace --summary-only` was flaky in this session.
- Treat the repo script as the measurement source of truth unless Ralph finds a concrete defect in that script itself.

## Product Reality

The game loop is already there. Ralph should not spend its first hours "making the game playable."

The high-value remaining work is:

- closing the last honest coverage gap, mostly in `game_app` and a little in `chess_core`
- polishing the player-facing shell so it reads less like an internal milestone build
- improving clarity and feel in the existing local-play flow without opening M4+ scope

Current visible roughness:

- board and pieces are still procedural and visually plain
- move feedback is mostly copy and subtle pulsing
- menu/pause/result/save-load flow works but still feels utilitarian
- promotion is keyboard-centric
- AI/audio seams exist but are currently no-ops

## Guardrails

- Do not expand scope into Stockfish/UCI gameplay, networking, installer/signing, controller support, or large asset-pipeline work.
- Do not game the coverage denominator.
- Keep behavior-heavy `game_app` files in scope.
- Keep `chess_core` authoritative.
- Keep `MatchSession` as the Bevy-facing bridge.
- Prefer extracting deterministic helper logic plus direct tests over piling on broad Bevy end-to-end tests.
- Preserve packaging/release paths even if smoke validation must be skipped for environment reasons.

## Recommended Ralph Order

1. Reconfirm the baseline with `cargo test --workspace` and the repo coverage script.
2. Attack the measured hotspot files first.
3. Expand tests only where they buy real branch coverage or protect important shell behavior.
4. Spend remaining time on user-facing shell polish inside the current local-play scope.
5. Finish with artifact-backed validation and an honest closeout.

## Operator Notes

- The easiest failure mode is to waste time fixing a coverage workflow that is already good enough when the real remaining problem is hotspot coverage.
- The second easiest failure mode is to chase pretty visuals while leaving `app.rs`, `board_coords.rs`, `app_shell_logic.rs`, `app_shell.rs`, and `chess_core/src/game.rs` below target.
- The best ROI is:
  - keep the repo script as the baseline measurement path
  - push `game_app` and `chess_core` above the line with targeted tests and small extractions
  - then improve shell clarity and feel without broadening scope
