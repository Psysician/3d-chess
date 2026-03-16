# Ralph Task List

Use this as the execution checklist for the long Ralph run.

## Phase 0: Baseline

- [x] Run `cargo fmt --all --check`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test --workspace`
- [ ] Run `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-baseline`
- [ ] Record the starting summary from `/tmp/3d-chess-coverage-baseline/summary.txt`
- [ ] Inspect `.github/workflows/ci.yml`, `tools/ci/coverage-workspace.sh`, and `tools/ci/parse_coverage.py` before changing coverage strategy

## Phase 1: Measured Coverage Hotspots

- [ ] Raise coverage in `crates/game_app/src/app.rs`
- [ ] Raise coverage in `crates/game_app/src/board_coords.rs`
- [ ] Raise coverage in `crates/game_app/src/plugins/app_shell_logic.rs`
- [ ] Raise coverage in `crates/game_app/src/plugins/app_shell.rs`
- [ ] Raise coverage in `crates/chess_core/src/game.rs`
- [ ] Keep exclusions narrow and documented
- [ ] Re-run the repo coverage script after each substantial batch and keep the artifact trail

## Phase 2: Secondary `game_app` Testability

- [ ] Review `crates/game_app/src/match_state.rs` for more pure decision logic that can be tested directly
- [ ] Review `crates/game_app/src/plugins/input.rs` for extractable branch-heavy interaction rules
- [ ] Review `crates/game_app/src/plugins/save_load.rs` for extractable persistence UI logic
- [ ] Review `crates/game_app/src/plugins/save_load_logic.rs` for remaining cheap direct-test gaps
- [ ] Extend the current helper-module pattern instead of pushing everything into integration tests

## Phase 3: Test Expansion

- [ ] Add direct tests for any newly extracted helper logic
- [ ] Expand focused tests around keyboard flow, overlay guards, selection clearing, promotion staging, and result-routing edges
- [ ] Expand save/load tests around recovery banner behavior, error aggregation, slot selection, destructive actions, and settings edge cases
- [ ] Keep only a small number of focused full-shell integration tests for critical flows
- [ ] Avoid replacing deterministic logic tests with broad Bevy E2E coverage

## Phase 4: Product-Shell Polish

- [ ] Tighten save/load, recovery, pause, setup, and result copy so it reads like player-facing UX instead of milestone/debug language
- [ ] Make menu/setup/pause/result transitions consistent and easier to reason about
- [ ] Check for stale selection, stale status text, stale overlays, or confusing confirmation states
- [ ] Improve keyboard affordances and promotion behavior where testing exposes rough edges
- [ ] Improve board/piece/HUD readability with low-risk changes
- [ ] Keep AI/audio seams quiet unless a real minimal implementation is landed

## Phase 5: Final Verification

- [ ] Re-run `cargo fmt --all --check`
- [ ] Re-run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Re-run `cargo test --workspace`
- [ ] Run `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-final`
- [ ] Confirm the final percentages from `/tmp/3d-chess-coverage-final/summary.txt`
- [ ] Run `cargo build --workspace --release`
- [ ] If feasible, run packaging and smoke-related commands that are still in scope

## Phase 6: Closeout

- [ ] Summarize exact files changed
- [ ] Summarize exact commands run and whether they passed
- [ ] Report exact final coverage numbers
- [ ] State the next 2-3 best follow-up tasks if `game_app` or `chess_core` still miss 90%
- [ ] Output `<promise>DONE</promise>` only if the prompt's completion criteria are actually satisfied
