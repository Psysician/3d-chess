You are working in `/home/franky/repos/3d-chess` on a Rust + Bevy desktop game workspace.

Your mission is to leave this repo in a more polished, more reliable, clearly playable local 3D chess state while driving honest automated coverage to a reproducible hard-90 path.

Work autonomously for multiple hours if needed. Make real code changes. Verify them. Keep going until you either hit the completion criteria or reach a concrete blocker you cannot solve locally.

## Verified Starting Point

- `cargo test --workspace` passes locally.
- The workspace has four crates:
  - `chess_core`
  - `chess_persistence`
  - `engine_uci`
  - `game_app`
- `chess_core` must remain the only gameplay/rules authority.
- `chess_persistence` owns save/load, recovery, settings, and on-disk contracts.
- `game_app` is the Bevy shell and already contains extraction seams like `app_shell_logic.rs` and `save_load_logic.rs`.
- The shipped scope is still the M3 local playable shell. Do not widen scope into M4+ roadmap work unless it directly unlocks the current mission.
- `AiMatchPlugin` and `ChessAudioPlugin` are still empty scaffolds. Do not pretend they are implemented.

## Coverage Ground Truth

Treat the repo script as the source of truth:

- `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-run`
- This was verified locally in this session and produced real summary artifacts.

Current honest baseline from `/tmp/3d-chess-coverage-run/summary.txt`:

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
- Do not spend time optimizing that command unless the repo script itself becomes unreliable.

## Hard Constraints

- Do not expand scope to Stockfish / real UCI gameplay, networking, installer signing, online play, controller support, or large asset-pipeline work.
- Do not game the coverage denominator. Keep behavior-heavy `game_app` files in scope. Narrow bootstrap-only exclusions are acceptable only if clearly justified.
- Preserve architecture boundaries:
  - `chess_core` stays pure and authoritative.
  - `chess_persistence` stays the file/recovery/settings boundary.
  - `game_app` stays a Bevy shell over `MatchSession`.
  - `engine_uci` remains the future AI seam.
- Prefer deterministic tests and extracted pure logic over brittle Bevy-heavy end-to-end expansion.
- Keep the repo green after each substantial batch:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`

## Primary Objectives

1. Keep the current local-play shell green and stable.
2. Push the measured hotspot files high enough that `game_app` and `chess_core` both clear 90% while keeping workspace coverage honest.
3. Improve the highest-value player-facing rough edges in `game_app` without widening product scope.
4. Leave the repo with artifact-backed coverage evidence and a credible hard-90 path.

## Read First

- `README.md`
- `plans/honest-90-coverage-plan.md`
- `crates/game_app/README.md`
- `crates/chess_core/src/game.rs`
- `crates/chess_core/tests/rules.rs`
- `crates/game_app/src/app.rs`
- `crates/game_app/src/board_coords.rs`
- `crates/game_app/src/match_state.rs`
- `crates/game_app/src/plugins/app_shell.rs`
- `crates/game_app/src/plugins/app_shell_logic.rs`
- `crates/game_app/src/plugins/input.rs`
- `crates/game_app/src/plugins/save_load.rs`
- `crates/game_app/src/plugins/save_load_logic.rs`
- `crates/game_app/src/plugins/move_feedback.rs`
- `crates/game_app/src/plugins/board_scene.rs`
- `crates/game_app/src/plugins/piece_view.rs`
- `tools/ci/coverage-workspace.sh`
- `tools/ci/README.md`

## Work Order

### Priority 0: Reconfirm baseline

- Re-run `cargo test --workspace`.
- Re-run the repo coverage script and capture the current summary numbers.

### Priority 1: Coverage hotspots first

- Attack these files in roughly this order:
  - `crates/game_app/src/app.rs`
  - `crates/game_app/src/board_coords.rs`
  - `crates/game_app/src/plugins/app_shell_logic.rs`
  - `crates/game_app/src/plugins/app_shell.rs`
  - `crates/chess_core/src/game.rs`
- Use small logic extractions where they improve both testability and code clarity.
- Add direct tests first; add Bevy/system/integration tests only where they protect real shell behavior.

### Priority 2: Secondary shell/testability work

- If needed after the measured hotspots, continue into:
  - `crates/game_app/src/match_state.rs`
  - `crates/game_app/src/plugins/input.rs`
  - `crates/game_app/src/plugins/save_load.rs`
  - `crates/game_app/src/plugins/save_load_logic.rs`
- Keep the current extraction pattern instead of stuffing more logic into giant Bevy systems.

### Priority 3: Product-shell polish

- Improve the quality of the existing shell, not the roadmap scope.
- Favor changes that make the current build feel more deliberate and complete:
  - clearer player-facing copy in setup, pause, result, save/load, and recovery surfaces
  - better selection / legal-target / last-move / check-state clarity
  - cleaner promotion flow and keyboard affordances
  - fewer stale or awkward shell states after transitions
  - low-risk readability upgrades in board/piece/HUD presentation
- Do not spend hours on speculative art work.
- Do not make AI/audio prominent unless you are implementing a real minimal slice.

### Priority 4: Final validation

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-final`
- `cargo build --workspace --release`
- Package/smoke only if feasible in the environment

## Rules

- Do not duplicate chess rules in Bevy systems.
- Do not widen coverage exclusions to make the percentage look good.
- Do not declare coverage tooling "broken" unless the repo script itself fails and you can show why.
- Do not declare 90% done unless the generated summary artifacts prove it.
- If honest 90% is not reachable in one session, stop at the highest truthful state and leave a clear ratchet-ready repo.

## Expected Deliverables

- Code changes that improve shell polish and/or testability.
- New or expanded tests, especially around the measured hotspot files.
- Fresh coverage artifacts with exact workspace and per-crate numbers.
- Updated docs if behavior or workflow changed materially.
- A final summary that includes:
  - what changed
  - exact commands run
  - exact coverage results from generated artifacts
  - remaining risks
  - next best follow-up if hard 90% was not reached

## Completion Criteria

Only output `<promise>DONE</promise>` when all of these are true:

- The repo is still green on fmt, clippy, and tests.
- You made meaningful progress on measured coverage hotspots and/or player-facing shell polish.
- You produced honest coverage artifacts from the repo script.
- The final summary names exact commands run and their outcomes.
