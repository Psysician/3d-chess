# Ralph Verification

Use this with `plans/ralph-prompt.md` and `plans/ralph-task-list.md`.

## Known Facts Before The Run

- `cargo test --workspace` passed locally during this session.
- Fresh verified coverage run:
  - command: `bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-run`
  - result: pass
- Fresh artifact-backed baseline from `/tmp/3d-chess-coverage-run/summary.txt`:
  - workspace: `90.09%`
  - chess_core: `89.95%`
  - chess_persistence: `95.49%`
  - engine_uci: `100.00%`
  - game_app: `89.29%`
- Direct `cargo llvm-cov --workspace --summary-only` was flaky in this session.

Interpretation:

- trust the repo coverage script
- treat direct one-off `cargo llvm-cov` invocations as secondary unless Ralph finds a concrete script defect

## Required Commands

### Core Gate

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Coverage Gate

Use the repo script:

```bash
bash tools/ci/coverage-workspace.sh "$PWD" /tmp/3d-chess-coverage-final
cat /tmp/3d-chess-coverage-final/summary.txt
```

### Release Gate

```bash
cargo build --workspace --release
```

### Optional Linux Packaging Smoke

Run only if the environment supports it:

```bash
bash tools/ci/package-game-app.sh "$PWD" /tmp/3d-chess-artifacts game_app-linux-x86_64
bash tools/ci/smoke-boot-linux.sh /tmp/3d-chess-artifacts/game_app-linux-x86_64.tar.gz
```

## Coverage Targets

Do not call the run complete unless the generated report proves at least:

- workspace `>= 90.00%`
- chess_core `>= 90.00%`
- chess_persistence `>= 95.00%`
- engine_uci `= 100.00%`
- game_app `>= 90.00%`

## Record Results Here

### Core Gate

- Result: pending

### Coverage Gate

- Result: pending
- Final report path: pending
- Final percentages: pending
- If failed, exact blocker: pending

### Release Gate

- Result: pending

### Optional Linux Packaging Smoke

- Result: pending
- If skipped, reason: pending

## Stop Rule

If honest coverage artifacts cannot be reproduced from the repo script, Ralph should still leave:

- the repo green on fmt, clippy, and tests
- the coverage blocker reduced to a precise root cause with evidence
- any useful surrounding testability or shell-polish improvements already landed
