# CI Notes

Portable artifact packaging, smoke-start verification, and coverage reporting for `game_app`.

## Coverage

- Run `bash tools/ci/coverage-workspace.sh` locally to generate raw and summarized coverage artifacts under `target/coverage`.
- The coverage job publishes `report.json`, `workspace.lcov`, `coverage-summary.json`, `summary.txt`, and `thresholds.env`.
- `COVERAGE_MODE=baseline` records the first honest report without broad exclusions or threshold failures.
- After the baseline run, copy the reported workspace and per-crate percentages into the `COVERAGE_*_THRESHOLD` GitHub variables and set `COVERAGE_MODE=non-regression`.
- When Milestones 2 and 3 land, promote the same variables to the agreed hard gate, including workspace `90` and ratcheted per-crate floors, then set `COVERAGE_MODE=hard-gate`.
- Keep exclusions narrow and documented. The shared script currently ignores only `crates/game_app/src/main.rs`, which is the bootstrap shim that just forwards into `game_app::run()`. Behavior-heavy `game_app` modules stay in scope.

## Architecture

- Packaging scripts stage the release binary with the runtime `assets/` tree into a single top-level app directory.
- Coverage runs through one shared `cargo-llvm-cov` script so local and CI measurement use the same denominator.
- Smoke scripts extract that directory and treat surviving a bounded startup window as proof of a bootable artifact. (ref: DL-006)

## Invariants

- CI proves packaged runtime boot, not just successful compilation. (ref: DL-006)
- Coverage ratchets from baseline to non-regression to hard-gate without swapping tools or broadening exclusions.
- Workspace totals and per-crate floors are both published so strong pure crates cannot hide weak `game_app` coverage.
- Windows and Linux archives stay self-contained with one top-level app directory so the workflow can upload runnable artifacts. (ref: DL-006)

## Runner Expectations

- Local Linux smoke prefers an already-working X11 display and falls back to `xvfb-run` when no live display is usable.
- Linux smoke uses `xvfb-run` to provide a stable desktop surface on hosted runners.
- Windows smoke starts the extracted executable directly from the staged app directory.
