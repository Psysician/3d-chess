# Plan

## Overview

Raise and enforce honest 90% coverage across the 3d-chess Rust workspace without gaming the denominator. The user directly asked for an honest enforceable 90% coverage plan; the rollout details below are plan recommendations inferred from the current repo state, especially missing coverage tooling and the game_app hotspot share of workspace lines.

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Use cargo-llvm-cov as the workspace coverage tool | Coverage enforcement spans Linux and Windows CI -> cargo-llvm-cov matches rustc/llvm instrumentation and workspace runs cleanly -> standardize coverage collection on cargo-llvm-cov instead of tarpaulin |
| DL-002 | Keep the coverage denominator honest with only narrow documented exclusions | game_app contains the largest share of workspace lines -> excluding app_shell or save_load would hide the dominant risk surface -> allow only true bootstrap or generated shims to be excluded and leave behavior-heavy files in scope |
| DL-003 | Reach 90% through per-crate logic coverage instead of Bevy-heavy end-to-end inflation | Pure crates and extracted shell helpers provide cheap deterministic branches -> broad Bevy end-to-end growth is brittle and slow -> buy most coverage from pure logic tests and keep a smaller set of shell flow integrations |
| DL-004 | Prioritize game_app hotspots by extracting and testing branchy shell logic | app_shell, save_load, input, and match_state dominate app-layer coverage risk -> their branch decisions are mostly pure or resource-driven helpers -> extract decision logic first and anchor only critical flows with integration tests |
| DL-005 | Roll CI enforcement from baseline to non-regression to hard 90% gates | The user asked for an honest enforceable 90% coverage plan, but did not specify the rollout sequence -> the repo currently has no coverage tooling or measured baseline -> choose a staged baseline -> non-regression -> hard-gate rollout as a plan recommendation grounded in repo state rather than an unspoken user requirement. |
| DL-006 | Use cargo-llvm-cov reports as the enforceable 90% signal for this rollout | The user asked for honest enforceable coverage, but not for a specific measurement stack -> repo analysis plus deepthink preferred cargo-llvm-cov for this workspace and CI shape -> treat cargo-llvm-cov as the plan-selected reporting source instead of leaving the metric source implicit. |
| DL-007 | Require measured game_app hotspot improvement before enabling hard workspace 90 gates | The user asked for honest 90% coverage without denominator gaming, but did not prescribe a hotspot gate -> game_app owns the largest share of workspace lines in the analyzed entry points -> make hotspot improvement an explicit plan recommendation so hard workspace 90 only turns on after app_shell save_load input and match_state show measured gains instead of riding pure-crate coverage. |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| Use cargo-tarpaulin as the primary coverage tool | The repo needs one coverage workflow that fits local runs plus Linux and Windows CI, and cargo-llvm-cov matches the existing rustc/LLVM toolchain path more cleanly. (ref: DL-001) |
| Flip CI straight to a hard workspace 90% gate immediately | The workspace does not have coverage tooling or the planned tests in place yet, so an immediate hard fail would block adoption instead of creating a credible ratchet. (ref: DL-005) |
| Exclude large game_app orchestration files from the denominator | app_shell and save_load are part of the dominant risk surface, so excluding them would game the metric instead of proving behavior. (ref: DL-002) |
| Buy most of the missing coverage with broader Bevy end-to-end smoke tests | Broad Bevy flows are slower and more brittle than direct logic tests, and they still miss many of the branch decisions that currently hold coverage down. (ref: DL-003) |
| Use `#[path]` includes with shim type redefinitions to test extracted logic without Bevy | Shim types can silently drift from real types, which undermines honest coverage. Instead, functions take primitive chess_core params where possible and the `test_support` module re-exports logic modules for integration tests using real types. (ref: DL-002, DL-003) |

### Constraints

- C-001 [workflow|source: AGENTS.md] Run the planner workflow fully for this task.
- C-002 [artifact-path|source: user] Write the final coverage planning artifact under plans/ with a coverage-specific filename rather than plans/test.md.
- C-003 [metric-honesty|source: user+deepthink] Keep the coverage denominator honest and do not broadly exclude behavior-heavy files to buy percentage.
- C-004 [tooling|source: deepthink] Prefer cargo-llvm-cov as the shared local and CI coverage tool for this Rust workspace.
- C-005 [rollout|source: deepthink] Ratchet enforcement from baseline to non-regression to hard workspace and per-crate gates instead of failing at 90 immediately.
- C-006 [test-strategy|source: deepthink] Do not rely mainly on brittle Bevy end-to-end tests to purchase coverage; extract pure logic wherever possible.

### Known Risks

- **Coverage enforcement drifts into denominator gaming through broad exclusions or missing per-crate visibility.**: Keep behavior-heavy files like app_shell and save_load in scope, publish per-crate reports, and document any narrow exclusions alongside the shared coverage script.
- **game_app hotspots can hold the workspace under 90 even if the pure crates approach complete coverage.**: Prioritize extraction and direct testing of app_shell, save_load, match_state, and input branches before flipping hard gates.
- **CI rollout friction can stall adoption if the first coverage job behaves like an immediate hard-fail migration.**: Ship baseline artifacts first, enforce non-regression next, and only turn on hard workspace and per-crate gates after the planned tests land.
- **A Bevy-heavy testing push could add slow brittle flows while still leaving branchy shell logic under-covered.**: Keep the shell integration suite small and buy most new coverage from extracted pure helpers plus deterministic crate-level tests.
- **Coverage CI runs Linux-only while the workspace tests on both Ubuntu and Windows.**: Bevy code can have platform-conditional branches that Linux-only coverage cannot measure. Keep the Windows test job as the behavioral safety net and revisit Windows coverage if platform-conditional branches grow.

## Invisible Knowledge

### System

Coverage in this plan is a proxy for exercised chess behavior, not a goal in itself; chess_core remains the rules authority, while game_app stays a thin Bevy shell whose branchy decisions should move into directly testable helpers instead of broader UI smoke coverage.

### Invariants

- Keep the denominator honest: app_shell, save_load, and other behavior-heavy files stay in coverage scope unless a narrow bootstrap-only exclusion is explicitly documented.
- Manual save and recovery/autosave are separate behaviors in both implementation and tests; do not collapse them into one generic persistence path.
- Per-crate ratchets matter because strong pure crates cannot be allowed to hide weak game_app coverage when the workspace gate reaches 90.

### Tradeoffs

- Prefer extracting pure save-load and app-shell logic for cheap deterministic branch coverage over chasing percentage with broader Bevy end-to-end tests.
- Prefer a baseline-to-hard-gate rollout over an immediate 90% fail-under so the repo can adopt honest enforcement without pretending the current baseline is already acceptable.

## Milestones

### Milestone 1: Coverage tooling and CI ratchet

**Files**: rust-toolchain.toml, .github/workflows/ci.yml, tools/ci/README.md, tools/ci/coverage-workspace.sh, tools/ci/parse_coverage.py

**Flags**: coverage, ci

**Requirements**:

- Add llvm-tools-preview to the pinned toolchain used for coverage collection
- Introduce one coverage script under tools/ci that runs cargo llvm-cov for workspace and per-crate summaries
- Add a Linux coverage job that installs cargo-llvm-cov and publishes baseline artifacts before the plan's recommended hard-gate phase

**Acceptance Criteria**:

- Coverage job produces machine-readable workspace and per-crate reports
- CI blocks coverage regressions against the recorded baseline threshold
- One config path raises the gate to workspace 90 and ratcheted per-crate floors after the planned test milestones land

**Tests**:

- Ubuntu coverage script run with cargo-llvm-cov installed
- Workflow upload of workspace and per-crate coverage artifacts
- Intentional threshold regression causes the coverage job to fail

#### Code Intent

- **CI-M-001-001** `rust-toolchain.toml`: Pin llvm-tools-preview alongside rustfmt and clippy so instrumented coverage runs use the same toolchain contract as local and CI builds. (refs: DL-001, DL-005)
- **CI-M-001-002** `tools/ci/coverage-workspace.sh`: Create one coverage entrypoint that installs stable cargo-llvm-cov expectations, runs workspace coverage, and emits report artifacts. Delegates per-crate threshold parsing to `tools/ci/parse_coverage.py` so the Python is independently testable and lintable. (refs: DL-001, DL-002, DL-005)
- **CI-M-001-005** `tools/ci/parse_coverage.py`: Standalone Python script that reads `cargo-llvm-cov` JSON reports, computes workspace and per-crate line-coverage percentages, writes summary artifacts, and enforces ratcheted thresholds. Kept separate from the shell script so report-format changes surface as parse errors instead of silent wrong percentages. (refs: DL-001, DL-002, DL-005)
- **CI-M-001-003** `.github/workflows/ci.yml`: Add an Ubuntu coverage job that installs cargo-llvm-cov, calls the shared coverage script, uploads reports, and enforces a ratchetable non-regression gate that later flips to workspace 90 plus per-crate floors. (refs: DL-001, DL-005)
- **CI-M-001-004** `tools/ci/README.md`: Document the local coverage command, the CI artifact contract, the narrow exclusion policy, and the ratchet path from baseline to enforced 90 coverage. (refs: DL-002, DL-005)

#### Code Changes

**CC-M-001-001** (rust-toolchain.toml) - implements CI-M-001-001

**Code:**

```diff
--- a/rust-toolchain.toml
+++ b/rust-toolchain.toml
@@ -1,4 +1,4 @@
 [toolchain]
 channel = "1.93.0"
-components = ["clippy", "rustfmt"]
+components = ["clippy", "llvm-tools-preview", "rustfmt"]
 profile = "minimal"

```

**Documentation:**

```diff
--- a/rust-toolchain.toml
+++ b/rust-toolchain.toml
@@ -1,3 +1,5 @@
+# llvm-tools-preview stays pinned with the workspace toolchain so cargo-llvm-cov
+# uses the same instrumentation components locally and in CI. (ref: DL-001) (ref: DL-006)
 [toolchain]
 channel = "1.93.0"
 components = ["clippy", "llvm-tools-preview", "rustfmt"]

```


**CC-M-001-002** (tools/ci/coverage-workspace.sh) - implements CI-M-001-002

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/coverage-workspace.sh
@@ -0,0 +1,95 @@
+#!/usr/bin/env bash
+set -euo pipefail
+
+workspace_root=${1:-$(pwd)}
+artifact_dir=${2:-"$workspace_root/target/coverage"}
+mode=${COVERAGE_MODE:-baseline}
+workspace_threshold=${COVERAGE_WORKSPACE_THRESHOLD:-0}
+chess_core_threshold=${COVERAGE_CHESS_CORE_THRESHOLD:-0}
+chess_persistence_threshold=${COVERAGE_CHESS_PERSISTENCE_THRESHOLD:-0}
+engine_uci_threshold=${COVERAGE_ENGINE_UCI_THRESHOLD:-0}
+game_app_threshold=${COVERAGE_GAME_APP_THRESHOLD:-0}
+cargo_llvm_cov_version=${CARGO_LLVM_COV_VERSION:-0.6.16}
+
+mkdir -p "$artifact_dir"
+
+if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
+  cargo install cargo-llvm-cov --locked --version "$cargo_llvm_cov_version"
+fi
+
+pushd "$workspace_root" >/dev/null
+cargo llvm-cov clean --workspace
+cargo llvm-cov test --workspace --json --output-path "$artifact_dir/workspace.json"
+cargo llvm-cov report --json --output-path "$artifact_dir/report.json"
+cargo llvm-cov report --lcov --output-path "$artifact_dir/workspace.lcov"
+
+cat > "$artifact_dir/thresholds.env" <<EOF
+COVERAGE_MODE=$mode
+COVERAGE_WORKSPACE_THRESHOLD=$workspace_threshold
+COVERAGE_CHESS_CORE_THRESHOLD=$chess_core_threshold
+COVERAGE_CHESS_PERSISTENCE_THRESHOLD=$chess_persistence_threshold
+COVERAGE_ENGINE_UCI_THRESHOLD=$engine_uci_threshold
+COVERAGE_GAME_APP_THRESHOLD=$game_app_threshold
+EOF
+
+python3 "$(dirname "$0")/parse_coverage.py" "$artifact_dir/report.json"
+popd >/dev/null
+
```

**Documentation:**

```diff
--- a/tools/ci/coverage-workspace.sh
+++ b/tools/ci/coverage-workspace.sh
@@ -1,4 +1,7 @@
 #!/usr/bin/env bash
+# One script owns baseline, non-regression, and hard-gate measurement so threshold
+# changes never change the instrumentation path or report format. (ref: DL-001) (ref: DL-005) (ref: DL-006)
+# Behavior-heavy game_app files stay in scope; only narrow documented exclusions belong here. (ref: DL-002)
 set -euo pipefail
 
 workspace_root=${1:-$(pwd)}

```


**CC-M-001-003** (.github/workflows/ci.yml) - implements CI-M-001-003

**Code:**

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -122,6 +122,52 @@
             pkg-config
       - name: Run test suite
         run: cargo test --workspace
+
+
+  coverage:
+    name: coverage (ubuntu)
+    runs-on: ubuntu-latest
+    needs: [fmt, clippy, test]
+    env:
+      COVERAGE_MODE: ${{ vars.COVERAGE_MODE || 'baseline' }}
+      COVERAGE_WORKSPACE_THRESHOLD: ${{ vars.COVERAGE_WORKSPACE_THRESHOLD || '0' }}
+      COVERAGE_CHESS_CORE_THRESHOLD: ${{ vars.COVERAGE_CHESS_CORE_THRESHOLD || '0' }}
+      COVERAGE_CHESS_PERSISTENCE_THRESHOLD: ${{ vars.COVERAGE_CHESS_PERSISTENCE_THRESHOLD || '0' }}
+      COVERAGE_ENGINE_UCI_THRESHOLD: ${{ vars.COVERAGE_ENGINE_UCI_THRESHOLD || '0' }}
+      COVERAGE_GAME_APP_THRESHOLD: ${{ vars.COVERAGE_GAME_APP_THRESHOLD || '0' }}
+    steps:
+      - uses: actions/checkout@v4
+      - name: Install Rust toolchain
+        uses: dtolnay/rust-toolchain@master
+        with:
+          toolchain: 1.93.0
+          components: llvm-tools-preview
+      - name: Cache cargo artifacts
+        uses: Swatinem/rust-cache@v2
+        with:
+          workspaces: . -> target
+      - name: Install Linux build dependencies
+        run: |
+          sudo apt-get update
+          sudo apt-get install -y \
+            libasound2-dev \
+            libudev-dev \
+            libwayland-dev \
+            libx11-dev \
+            libxcursor-dev \
+            libxi-dev \
+            libxinerama-dev \
+            libxkbcommon-dev \
+            libxrandr-dev \
+            libxxf86vm-dev \
+            pkg-config
+      - name: Run workspace coverage
+        run: bash tools/ci/coverage-workspace.sh "$GITHUB_WORKSPACE" "$RUNNER_TEMP/coverage"
+      - name: Upload coverage artifacts
+        uses: actions/upload-artifact@v4
+        with:
+          name: coverage-report
+          path: ${{ runner.temp }}/coverage
 
   build-release:
     name: build-release (${{ matrix.os }})

```

**Documentation:**

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -124,6 +124,9 @@
         run: cargo test --workspace
 
 
+  # Coverage ratchets through the same workspace script in baseline, non-regression,
+  # and hard-gate modes so CI policy changes without changing the metric source. (ref: DL-005) (ref: DL-006)
+  # Per-crate thresholds stay visible because workspace 90 alone must not hide a weak game_app. (ref: DL-002) (ref: DL-007)
   coverage:
     name: coverage (ubuntu)
     runs-on: ubuntu-latest

```


**CC-M-001-004** (tools/ci/README.md) - implements CI-M-001-004

**Code:**

```diff
--- a/tools/ci/README.md
+++ b/tools/ci/README.md
@@ -1,15 +1,26 @@
-# CI Packaging Notes
+# CI Notes

-Portable artifact packaging and smoke-start verification for `game_app`.
+Portable artifact packaging, smoke-start verification, and coverage reporting for `game_app`.
+
+## Coverage
+
+- Run `bash tools/ci/coverage-workspace.sh` locally to generate workspace and per-crate reports under `target/coverage`.
+- The GitHub Actions coverage job publishes raw `cargo-llvm-cov` JSON, LCOV output, summary text, and the exact threshold snapshot used for that run.
+- Baseline mode records the first report without broad exclusions so the denominator stays honest.
+- Non-regression mode reuses the same script and raises `COVERAGE_*_THRESHOLD` values to the recorded floors.
+- Hard-gate mode keeps the same measurement path and flips those env vars to workspace `90` plus the agreed per-crate floors once the hotspot milestones land.
+- Keep exclusions narrow and documented; bootstrap shims such as `main.rs` can be considered, but `app_shell.rs`, `save_load.rs`, `input.rs`, and `match_state.rs` stay in scope.

 ## Architecture

 - Packaging scripts stage the release binary with the runtime `assets/` tree into a single top-level app directory.
+- Coverage scripts stage one shared `cargo-llvm-cov` entrypoint so local runs and CI enforce the same thresholds.
 - Smoke scripts extract that directory and treat surviving a bounded startup window as proof of a bootable artifact. (ref: DL-006)

 ## Invariants

 - CI proves packaged runtime boot, not just successful compilation. (ref: DL-006)
+- CI coverage uses the same workspace script in baseline, non-regression, and hard-gate modes so threshold changes do not change the denominator.
 - Windows and Linux archives stay self-contained with one top-level app directory so the workflow can upload runnable artifacts. (ref: DL-006)

 ## Runner Expectations

```

**Documentation:**

```diff
--- a/tools/ci/README.md
+++ b/tools/ci/README.md
@@ -3,6 +3,9 @@
 Portable artifact packaging, smoke-start verification, and coverage reporting for `game_app`.
 
 ## Coverage
+
+- Baseline, non-regression, and hard-gate modes keep the same `cargo-llvm-cov` script so threshold changes never swap the metric source. (ref: DL-005) (ref: DL-006)
+- Per-crate floors stay visible beside the workspace total so strong pure crates cannot hide weak `game_app` coverage. (ref: DL-002) (ref: DL-007)
 
 - Run `bash tools/ci/coverage-workspace.sh` locally to generate workspace and per-crate reports under `target/coverage`.
 - The GitHub Actions coverage job publishes raw `cargo-llvm-cov` JSON, LCOV output, summary text, and the exact threshold snapshot used for that run.

```


**CC-M-001-005** (tools/ci/parse_coverage.py) - implements CI-M-001-005

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/parse_coverage.py
@@ -0,0 +1,68 @@
+#!/usr/bin/env python3
+"""Parse cargo-llvm-cov JSON reports into workspace and per-crate summaries.
+
+Kept as a standalone script so report-format changes surface as parse errors
+instead of silent wrong percentages. (ref: DL-001) (ref: DL-005)
+"""
+
+import json
+import os
+import sys
+
+
+def load_report(path: str) -> dict:
+    with open(path, encoding='utf-8') as handle:
+        report = json.load(handle)
+    data = report.get('data')
+    if isinstance(data, list) and data:
+        return data[0]
+    return report
+
+
+def line_counts(bucket: dict) -> tuple[int, int]:
+    lines = bucket.get('lines', {})
+    return int(lines.get('covered', 0)), int(lines.get('count', 0))
+
+
+def main() -> None:
+    report_path = sys.argv[1]
+    report_root = load_report(report_path)
+    workspace_covered, workspace_count = line_counts(report_root.get('totals', {}))
+    counts = {
+        'workspace': [workspace_covered, workspace_count],
+        'chess_core': [0, 0],
+        'chess_persistence': [0, 0],
+        'engine_uci': [0, 0],
+        'game_app': [0, 0],
+    }
+
+    for entry in report_root.get('files', []):
+        filename = entry.get('filename', '')
+        covered, count = line_counts(entry.get('summary', {}))
+        for crate in ('chess_core', 'chess_persistence', 'engine_uci', 'game_app'):
+            if f'/crates/{crate}/' in filename or filename.startswith(f'crates/{crate}/'):
+                counts[crate][0] += covered
+                counts[crate][1] += count
+
+    summary = {
+        key: 0.0 if count == 0 else round((covered / count) * 100, 2)
+        for key, (covered, count) in counts.items()
+    }
+
+    output_dir = os.path.dirname(report_path)
+    with open(os.path.join(output_dir, 'coverage-summary.json'), 'w', encoding='utf-8') as handle:
+        json.dump({'mode': os.environ.get('COVERAGE_MODE', 'baseline'), 'summary': summary}, handle, indent=2)
+
+    with open(os.path.join(output_dir, 'summary.txt'), 'w', encoding='utf-8') as handle:
+        for name in ('workspace', 'chess_core', 'chess_persistence', 'engine_uci', 'game_app'):
+            covered, count = counts[name]
+            handle.write(f'{name}: {summary[name]:.2f}% ({covered}/{count})\n')
+
+    thresholds = {
+        'workspace': float(os.environ.get('COVERAGE_WORKSPACE_THRESHOLD', '0')),
+        'chess_core': float(os.environ.get('COVERAGE_CHESS_CORE_THRESHOLD', '0')),
+        'chess_persistence': float(os.environ.get('COVERAGE_CHESS_PERSISTENCE_THRESHOLD', '0')),
+        'engine_uci': float(os.environ.get('COVERAGE_ENGINE_UCI_THRESHOLD', '0')),
+        'game_app': float(os.environ.get('COVERAGE_GAME_APP_THRESHOLD', '0')),
+    }
+    violations = [f"{name} {summary[name]:.2f}% < {thresholds[name]:.2f}%" for name in thresholds if summary[name] < thresholds[name]]
+    if violations and os.environ.get('COVERAGE_MODE', 'baseline') != 'baseline':
+        raise SystemExit('coverage threshold failures: ' + '; '.join(violations))
+
+
+if __name__ == '__main__':
+    main()

```


### Milestone 2: Pure crate coverage uplift

**Files**: crates/chess_core/src/game.rs, crates/chess_core/tests/rules.rs, crates/chess_persistence/src/lib.rs, crates/chess_persistence/src/store.rs, crates/chess_persistence/tests/session_store.rs, crates/engine_uci/src/controller.rs, crates/engine_uci/src/mock.rs, crates/engine_uci/tests/controller.rs

**Flags**: coverage, tests

**Requirements**:

- Cover chess_core parse and move error branches without weakening scenario tests
- Cover SessionStore unhappy paths and settings edge cases with real filesystem tests
- Cover engine_uci request validation and trait behavior with direct controller tests

**Acceptance Criteria**:

- chess_core reaches near-complete rule and error-path coverage
- chess_persistence covers manual save recovery and settings failure paths
- engine_uci reaches near-complete deterministic controller coverage

**Tests**:

- rules.rs adds invalid FEN and illegal move scenarios
- session_store.rs adds corrupt settings and recovery error scenarios
- engine_uci controller tests cover empty position and scripted bestmove responses

#### Code Intent

- **CI-M-002-001** `crates/chess_core/src/game.rs::GameState::from_fen`: Keep malformed FEN branches individually observable so tests can assert board side castling en-passant and move-counter failures without collapsing them into one generic parse error. (refs: DL-003)
- **CI-M-002-002** `crates/chess_persistence/src/lib.rs`: Expand snapshot roundtrip assertions so legal move sets metadata and shell-state fields stay intact across more persisted-session shapes. (refs: DL-003)
- **CI-M-002-003** `crates/chess_core/tests/rules.rs`: Extend rules regression coverage to illegal move errors finished-game rejection malformed FEN fixtures and repetition bookkeeping edges while keeping tests behavior-oriented. (refs: DL-003)
- **CI-M-002-004** `crates/chess_persistence/src/store.rs`: Surface deterministic error handling for corrupt settings recovery files and save-index reads so repository failures stay explicit and do not silently reset known-good state. (refs: DL-003)
- **CI-M-002-005** `crates/engine_uci/tests/controller.rs`: Add direct controller tests that lock request validation error display and scripted bestmove responses without introducing engine-process complexity. (refs: DL-003)
- **CI-M-002-006** `crates/chess_persistence/tests/session_store.rs`: Add real-filesystem coverage for corrupt JSON missing settings fallback invalid slot persistence and recovery clear failures using temporary roots instead of mocks. (refs: DL-003)
- **CI-M-002-007** `crates/engine_uci/src/controller.rs`: Keep controller validation and bestmove/result mapping branches directly observable so deterministic tests can cover request parsing, error display, and response translation without engine-process coupling. (refs: DL-003)
- **CI-M-002-008** `crates/engine_uci/src/mock.rs`: Extend the mock engine seam with scripted success and failure responses that let controller tests cover deterministic UCI interactions without spawning a real engine process. (refs: DL-003)

#### Code Changes

**CC-M-002-001** (crates/chess_core/src/game.rs) - implements CI-M-002-001

**Code:**

```diff
--- a/crates/chess_core/src/game.rs
+++ b/crates/chess_core/src/game.rs
@@ -271,23 +271,12 @@ impl GameState {
         Self::validate_king_count(&board, Side::White)?;
         Self::validate_king_count(&board, Side::Black)?;

-        let side_to_move = match fields[1] {
-            "w" => Side::White,
-            "b" => Side::Black,
-            _ => return Err(FenError::InvalidSideToMove),
-        };
+        let side_to_move = Self::parse_side_to_move(fields[1])?;

         let castling_rights = Self::parse_castling_rights(fields[2])?;
         let en_passant_target = Self::parse_en_passant_target(fields[3])?;
-        let halfmove_clock = fields[4]
-            .parse::<u16>()
-            .map_err(|_| FenError::InvalidHalfmoveClock)?;
-        let fullmove_number = fields[5]
-            .parse::<u16>()
-            .map_err(|_| FenError::InvalidFullmoveNumber)?;
-        if fullmove_number == 0 {
-            return Err(FenError::InvalidFullmoveNumber);
-        }
+        let (halfmove_clock, fullmove_number) =
+            Self::parse_move_counters(fields[4], fields[5])?;

         Ok(Self::from_parts(
             board,
@@ -301,6 +294,27 @@ impl GameState {
         ))
     }

+    fn parse_side_to_move(field: &str) -> Result<Side, FenError> {
+        match field {
+            "w" => Ok(Side::White),
+            "b" => Ok(Side::Black),
+            _ => Err(FenError::InvalidSideToMove),
+        }
+    }
+
+    fn parse_move_counters(halfmove_clock: &str, fullmove_number: &str) -> Result<(u16, u16), FenError> {
+        let halfmove_clock = halfmove_clock
+            .parse::<u16>()
+            .map_err(|_| FenError::InvalidHalfmoveClock)?;
+        let fullmove_number = fullmove_number
+            .parse::<u16>()
+            .map_err(|_| FenError::InvalidFullmoveNumber)?;
+        if fullmove_number == 0 {
+            return Err(FenError::InvalidFullmoveNumber);
+        }
+        Ok((halfmove_clock, fullmove_number))
+    }
+
     #[must_use]
     pub fn current_position_repetition_count(&self) -> usize {
         let current = self.current_position_key();

```

**Documentation:**

```diff
--- a/crates/chess_core/src/game.rs
+++ b/crates/chess_core/src/game.rs
@@ -1,3 +1,4 @@
+//! FEN parsing helpers keep invalid-token branches distinct so rule coverage comes from deterministic domain logic instead of shell smoke tests. (ref: DL-003)
 use std::fmt::{Display, Formatter};
 
 use serde::{Deserialize, Serialize};

```


**CC-M-002-002** (crates/chess_persistence/src/lib.rs) - implements CI-M-002-002

**Code:**

```diff
--- a/crates/chess_persistence/src/lib.rs
+++ b/crates/chess_persistence/src/lib.rs
@@ -58,6 +58,16 @@
             serde_json::from_str(&encoded).expect("deserializing the snapshot should succeed");
         let restored = decoded.restore_game_state();
 
+        assert_eq!(
+            decoded.shell_state(),
+            &SnapshotShellState {
+                selected_square: Some(c5),
+                pending_promotion: Some(PendingPromotionSnapshot { from: e2, to: e4 }),
+                last_move: Some(Move::new(c7, c5)),
+                claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
+                dirty_recovery: true,
+            }
+        );
         assert_eq!(decoded.version, SaveFormatVersion::V2);
         assert_eq!(decoded.metadata(), &metadata);
         assert_eq!(decoded.shell_state().selected_square, Some(c5));
@@ -76,4 +86,33 @@
         assert_eq!(restored.legal_moves(), after_c5.legal_moves());
         assert!(matches!(restored.status(), GameStatus::Ongoing { .. }));
     }
+
+#[test]
+fn recovery_snapshot_roundtrip_keeps_resume_metadata_and_shell_flags() {
+    let game_state = GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1")
+        .expect("fixture FEN should parse");
+    let snapshot = GameSnapshot::from_parts(
+        game_state.clone(),
+        SnapshotMetadata {
+            label: String::from("recovery"),
+            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
+            updated_at_utc: Some(String::from("2026-03-15T00:01:00Z")),
+            notes: Some(String::from("Interrupted session")),
+            save_kind: SaveKind::Recovery,
+            session_id: String::from("recovery"),
+            recovery_key: Some(String::from("autosave")),
+        },
+        SnapshotShellState {
+            dirty_recovery: true,
+            ..SnapshotShellState::default()
+        },
+    );
+
+    let restored = snapshot.restore_game_state();
+    assert_eq!(snapshot.metadata().save_kind, SaveKind::Recovery);
+    assert_eq!(snapshot.metadata().recovery_key.as_deref(), Some("autosave"));
+    assert!(snapshot.shell_state().dirty_recovery);
+    assert_eq!(restored.to_fen(), game_state.to_fen());
+    assert_eq!(restored.legal_moves(), game_state.legal_moves());
 }
+}

```

**Documentation:**

```diff
--- a/crates/chess_persistence/src/lib.rs
+++ b/crates/chess_persistence/src/lib.rs
@@ -1,3 +1,4 @@
+//! Snapshot roundtrips preserve shell metadata and recovery flags so persistence coverage proves the contract shared with `game_app`. (ref: DL-003)
 pub mod snapshot;
 pub mod store;
 

```


**CC-M-002-003** (crates/chess_core/tests/rules.rs) - implements CI-M-002-003

**Code:**

```diff
--- a/crates/chess_core/tests/rules.rs
+++ b/crates/chess_core/tests/rules.rs
@@ -1,6 +1,6 @@
 use chess_core::{
-    AutomaticDrawReason, DrawReason, GameOutcome, GameState, GameStatus, Move, Piece, PieceKind,
-    Side, Square, WinReason,
+    AutomaticDrawReason, DrawReason, FenError, GameOutcome, GameState, GameStatus, Move,
+    MoveError, Piece, PieceKind, Side, Square, WinReason,
 };
 
 fn square(name: &str) -> Square {
@@ -200,3 +200,40 @@
         )))
     );
 }
+
+
+#[test]
+fn from_fen_surfaces_invalid_tokens_as_distinct_errors() {
+    assert!(matches!(
+        GameState::from_fen("8/8/8/8/8/8/8/8 x - - 0 1"),
+        Err(FenError::InvalidSideToMove)
+    ));
+    assert!(matches!(
+        GameState::from_fen("8/8/8/8/8/8/8/8 w - z9 0 1"),
+        Err(FenError::InvalidEnPassantTarget)
+    ));
+    assert!(matches!(
+        GameState::from_fen("8/8/8/8/8/8/8/8 w - - 0 0"),
+        Err(FenError::InvalidFullmoveNumber)
+    ));
+}
+
+#[test]
+fn apply_move_reports_wrong_side_illegal_and_finished_branches() {
+    let start = GameState::starting_position();
+    assert_eq!(
+        start.apply_move(Move::new(square("e7"), square("e5"))),
+        Err(MoveError::WrongSideToMove)
+    );
+    assert_eq!(
+        start.apply_move(Move::new(square("e2"), square("e5"))),
+        Err(MoveError::IllegalMove)
+    );
+
+    let finished =
+        GameState::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("FEN should parse");
+    assert_eq!(
+        finished.apply_move(Move::new(square("h8"), square("h7"))),
+        Err(MoveError::GameAlreadyFinished)
+    );
+}

```

**Documentation:**

```diff
--- a/crates/chess_core/tests/rules.rs
+++ b/crates/chess_core/tests/rules.rs
@@ -1,3 +1,4 @@
+//! These cases keep parser and move-validation failures distinct so rule coverage continues to come from deterministic domain logic instead of shell smoke tests. (ref: DL-003)
 use chess_core::{
     AutomaticDrawReason, DrawReason, FenError, GameOutcome, GameState, GameStatus, Move,
     MoveError, Piece, PieceKind, Side, Square, WinReason,

```


**CC-M-002-004** (crates/chess_persistence/src/store.rs) - implements CI-M-002-004

**Code:**

```diff
--- a/crates/chess_persistence/src/store.rs
+++ b/crates/chess_persistence/src/store.rs
@@ -251,12 +251,9 @@ impl SessionStore {
     }
 
     pub fn load_settings(&self) -> StoreResult<ShellSettings> {
-        let path = self.settings_file();
-        if !path.exists() {
-            return Ok(ShellSettings::default());
-        }
-
-        self.read_json(&path)
+        Ok(self
+            .read_optional_json::<ShellSettings>(&self.settings_file())?
+            .unwrap_or_default())
     }
 
     pub fn save_settings(&self, settings: &ShellSettings) -> StoreResult<()> {
@@ -315,6 +312,16 @@ impl SessionStore {
     fn read_json<T: DeserializeOwned>(&self, path: &Path) -> StoreResult<T> {
         let bytes = fs::read(path)?;
         Ok(serde_json::from_slice(&bytes)?)
+    }
+
+    fn read_optional_json<T: DeserializeOwned>(&self, path: &Path) -> StoreResult<Option<T>> {
+        if !path.exists() {
+            return Ok(None);
+        }
+
+        let bytes = fs::read(path)?;
+        let decoded = serde_json::from_slice(&bytes)?;
+        Ok(Some(decoded))
     }
 
     fn write_json_atomic<T: Serialize>(&self, path: &Path, value: &T) -> StoreResult<()> {

```

**Documentation:**

```diff
--- a/crates/chess_persistence/src/store.rs
+++ b/crates/chess_persistence/src/store.rs
@@ -1,5 +1,6 @@
 //! File-backed repository for manual saves, interrupted-session recovery, and the shipped shell settings trio.
 //! The repository owns platform paths and atomic I/O so gameplay code only exchanges snapshots.
+//! Optional settings reads stay tolerant of absence, while recovery and manual-save paths preserve explicit I/O and decode failures for honest persistence coverage. (ref: DL-003)
 
 use std::fmt::{Display, Formatter};
 use std::fs;

```


**CC-M-002-005** (crates/engine_uci/tests/controller.rs) - implements CI-M-002-005

**Code:**

```diff
--- /dev/null
+++ b/crates/engine_uci/tests/controller.rs
@@ -0,0 +1,47 @@
+use engine_uci::{EngineController, EngineRequest, MockEngineController};
+
+#[test]
+fn request_validation_rejects_blank_positions_and_zero_movetime() {
+    let blank = EngineRequest::new("   ", 150)
+        .validate()
+        .expect_err("blank positions should be rejected");
+    assert_eq!(blank.to_string(), "position_notation must not be empty");
+
+    let zero = EngineRequest::new("startpos", 0)
+        .validate()
+        .expect_err("zero movetime should be rejected");
+    assert_eq!(zero.to_string(), "movetime_millis must be greater than zero");
+}
+
+#[test]
+fn mock_controller_returns_scripted_bestmove_for_valid_requests() {
+    let mut controller = MockEngineController::new("e2e4");
+    let response = controller
+        .evaluate(&EngineRequest::new("startpos", 150))
+        .expect("mock engine should answer valid requests");
+
+    assert_eq!(response.bestmove_uci.as_deref(), Some("e2e4"));
+    assert!(response.info.contains("startpos"));
+}
+
+#[test]
+fn mock_controller_can_surface_health_and_scripted_failures() {
+    let mut unhealthy = MockEngineController::new("e2e4").with_health(false);
+    assert!(!unhealthy.is_healthy());
+    assert_eq!(
+        unhealthy
+            .evaluate(&EngineRequest::new("startpos", 150))
+            .expect_err("unhealthy mock should fail")
+            .to_string(),
+        "mock engine is unhealthy"
+    );
+
+    let mut failing = MockEngineController::new("e2e4").with_failure("uci unavailable");
+    assert_eq!(
+        failing
+            .evaluate(&EngineRequest::new("startpos", 150))
+            .expect_err("configured failure should surface")
+            .to_string(),
+        "uci unavailable"
+    );
+}

```

**Documentation:**

```diff
--- a/crates/engine_uci/tests/controller.rs
+++ b/crates/engine_uci/tests/controller.rs
@@ -1,3 +1,4 @@
+//! The controller suite proves validation, health, and scripted bestmove behavior directly so the crate reaches its floor without process-management noise. (ref: DL-003)
 use engine_uci::{EngineController, EngineRequest, MockEngineController};
 
 #[test]

```


**CC-M-002-006** (crates/chess_persistence/tests/session_store.rs) - implements CI-M-002-006

**Code:**

```diff
--- a/crates/chess_persistence/tests/session_store.rs
+++ b/crates/chess_persistence/tests/session_store.rs
@@ -214,3 +214,44 @@
     assert_eq!(loaded.metadata().session_id, "tampered-slot");
     assert_eq!(loaded.metadata().save_kind, SaveKind::Manual);
 }
+
+
+#[test]
+fn corrupt_settings_file_returns_serialization_error_instead_of_defaulting() {
+    let temp_dir = TempDir::new().expect("temp dir should be created");
+    let store = SessionStore::new(temp_dir.path());
+    fs::write(temp_dir.path().join("settings.json"), b"{not-json")
+        .expect("corrupt settings fixture should be written");
+
+    let error = store
+        .load_settings()
+        .expect_err("corrupt settings should stay visible to callers");
+    assert!(matches!(error, StoreError::Serialization(_)));
+}
+
+#[test]
+fn corrupt_recovery_file_surfaces_error_before_resume_logic_uses_it() {
+    let temp_dir = TempDir::new().expect("temp dir should be created");
+    let store = SessionStore::new(temp_dir.path());
+    fs::create_dir_all(temp_dir.path().join("recovery")).expect("recovery directory should exist");
+    fs::write(temp_dir.path().join("recovery").join("current.json"), b"{not-json")
+        .expect("corrupt recovery fixture should be written");
+
+    let error = store
+        .load_recovery()
+        .expect_err("corrupt recovery data should not be hidden");
+    assert!(matches!(error, StoreError::Serialization(_)));
+}
+
+#[test]
+fn clear_recovery_reports_io_failures_when_path_is_a_directory() {
+    let temp_dir = TempDir::new().expect("temp dir should be created");
+    let store = SessionStore::new(temp_dir.path());
+    fs::create_dir_all(temp_dir.path().join("recovery").join("current.json"))
+        .expect("directory-backed recovery fixture should exist");
+
+    let error = store
+        .clear_recovery()
+        .expect_err("directory-backed recovery path should fail to clear");
+    assert!(matches!(error, StoreError::Io(_)));
+}

```

**Documentation:**

```diff
--- a/crates/chess_persistence/tests/session_store.rs
+++ b/crates/chess_persistence/tests/session_store.rs
@@ -1,3 +1,4 @@
+//! These repository tests exercise corrupt-data and directory-backed failures directly so crate coverage comes from persistence behavior, not shell-side retries. (ref: DL-003)
 use std::fs;
 use std::io;
 

```


**CC-M-002-007** (crates/engine_uci/src/controller.rs) - implements CI-M-002-007

**Code:**

```diff
--- a/crates/engine_uci/src/controller.rs
+++ b/crates/engine_uci/src/controller.rs
@@ -11,6 +11,17 @@ impl EngineRequest {
             position_notation: position_notation.into(),
             movetime_millis,
         }
+    }
+
+    pub fn validate(&self) -> Result<(), EngineError> {
+        if self.position_notation.trim().is_empty() {
+            return Err(EngineError::new("position_notation must not be empty"));
+        }
+        if self.movetime_millis == 0 {
+            return Err(EngineError::new("movetime_millis must be greater than zero"));
+        }
+
+        Ok(())
     }
 }

@@ -21,5 +32,15 @@ pub struct EngineResponse {
     pub info: String,
 }
+
+impl EngineResponse {
+    #[must_use]
+    pub fn bestmove(bestmove_uci: impl Into<String>, info: impl Into<String>) -> Self {
+        Self {
+            bestmove_uci: Some(bestmove_uci.into()),
+            info: info.into(),
+        }
+    }
+}

 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct EngineError {

```

**Documentation:**

```diff
--- a/crates/engine_uci/src/controller.rs
+++ b/crates/engine_uci/src/controller.rs
@@ -1,3 +1,4 @@
+//! Validation and response constructors stay pure so controller coverage reaches near-complete line coverage without spawning a real engine process. (ref: DL-003)
 use std::fmt::{Display, Formatter};
 
 #[derive(Debug, Clone, PartialEq, Eq)]

```


**CC-M-002-008** (crates/engine_uci/src/mock.rs) - implements CI-M-002-008

**Code:**

```diff
--- a/crates/engine_uci/src/mock.rs
+++ b/crates/engine_uci/src/mock.rs
@@ -3,15 +3,29 @@ use crate::{EngineController, EngineError, EngineRequest, EngineResponse};
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct MockEngineController {
     scripted_move: String,
+    scripted_error: Option<String>,
     healthy: bool,
 }

 impl MockEngineController {
     #[must_use]
     pub fn new(scripted_move: impl Into<String>) -> Self {
         Self {
             scripted_move: scripted_move.into(),
+            scripted_error: None,
             healthy: true,
         }
+    }
+
+    #[must_use]
+    pub fn with_health(mut self, healthy: bool) -> Self {
+        self.healthy = healthy;
+        self
+    }
+
+    #[must_use]
+    pub fn with_failure(mut self, message: impl Into<String>) -> Self {
+        self.scripted_error = Some(message.into());
+        self
     }
 }
@@ -29,17 +42,18 @@ impl EngineController for MockEngineController {
         self.healthy
     }

     fn evaluate(&mut self, request: &EngineRequest) -> Result<EngineResponse, EngineError> {
-        if request.position_notation.trim().is_empty() {
-            return Err(EngineError::new("position_notation must not be empty"));
+        request.validate()?;
+        if !self.healthy {
+            return Err(EngineError::new("mock engine is unhealthy"));
+        }
+        if let Some(message) = self.scripted_error.clone() {
+            return Err(EngineError::new(message));
         }

-        Ok(EngineResponse {
-            bestmove_uci: Some(self.scripted_move.clone()),
-            info: format!(
-                "mock evaluation for '{}' at {} ms",
-                request.position_notation, request.movetime_millis
-            ),
-        })
+        Ok(EngineResponse::bestmove(
+            self.scripted_move.clone(),
+            format!("mock evaluation for '{}' at {} ms", request.position_notation, request.movetime_millis),
+        ))
     }
 }

```

**Documentation:**

```diff
--- a/crates/engine_uci/src/mock.rs
+++ b/crates/engine_uci/src/mock.rs
@@ -1,3 +1,4 @@
+//! The mock controller scripts health and failure branches deterministically so UCI coverage does not depend on process variability. (ref: DL-003)
 use crate::{EngineController, EngineError, EngineRequest, EngineResponse};
 
 #[derive(Debug, Clone, PartialEq, Eq)]

```


### Milestone 3: game_app logic extraction and shell coverage

**Files**: crates/game_app/src/lib.rs, crates/game_app/src/match_state.rs, crates/game_app/src/plugins/mod.rs, crates/game_app/src/plugins/input.rs, crates/game_app/src/plugins/save_load.rs, crates/game_app/src/plugins/save_load_logic.rs, crates/game_app/src/plugins/app_shell.rs, crates/game_app/src/plugins/app_shell_logic.rs, crates/game_app/tests/local_match_flow.rs, crates/game_app/tests/match_state_flow.rs, crates/game_app/tests/save_load_flow.rs, crates/game_app/tests/promotion_flow.rs, crates/game_app/tests/app_shell_logic.rs, crates/game_app/tests/save_load_logic.rs

**Flags**: coverage, refactor, bevy

**Requirements**:

- Extract branch-heavy app_shell and save_load decisions into pure helpers that keep Bevy spawn glue thin
- Expand match_state and input coverage around snapshot conversion draw claims recovery dirtiness and overlay guards
- Keep a small shell integration suite for launch load resume promotion and result flows instead of chasing percentage with more end-to-end scenes

**Acceptance Criteria**:

- Listed game_app hotspots gain direct coverage for extracted logic and critical shell flows with remaining gaps limited to thin Bevy wiring
- Workspace aggregate reaches honest 90 after combining pure-crate and game_app work
- CI can switch from the plan's non-regression phase to hard workspace and per-crate gates without exclusions for app_shell or save_load

**Tests**:

- Direct tests cover app shell labels status copy button routing and recovery policy cycling
- Direct tests cover save-load request handling recovery banner state and autosave dirtiness transitions
- Integration flows keep local match manual load quick save resume recovery and promotion behavior green

#### Code Intent

- **CI-M-003-001** `crates/game_app/src/match_state.rs`: Add direct coverage for snapshot restore replacement summary draw-claim and recovery-dirty transitions so MatchSession stays the authoritative shell bridge under fast tests. (refs: DL-003, DL-004)
- **CI-M-003-002** `crates/game_app/src/plugins/input.rs`: Expand direct system tests for square deselection promotion staging overlay guards quick-save gating and keyboard cancel behavior without growing full-scene end-to-end coverage. (refs: DL-003, DL-004)
- **CI-M-003-003** `crates/game_app/src/plugins/save_load_logic.rs`: Extract pure save-load helper logic for banner visibility error aggregation label selection and ratchet-ready threshold messages so repository branch coverage does not depend on Bevy app setup. (refs: DL-002, DL-003, DL-004, DL-005)
- **CI-M-003-004** `crates/game_app/src/plugins/save_load.rs`: Thin the Bevy persistence systems down to repository I/O wiring and use focused tests plus existing flow integrations to cover manual save load resume clear and autosave branches honestly. (refs: DL-002, DL-003, DL-004)
- **CI-M-003-005** `crates/game_app/src/plugins/app_shell_logic.rs`: Extract pure app-shell helpers for result titles status copy confirmation text save selection state and recovery policy labels so the large menu surface gains cheap branch coverage. Functions that previously took `&MatchSession` take primitive chess_core params instead (`GameStatus`, `Option<ClaimedDrawReason>`, `Option<Move>`) so tests exercise real types without shims. (refs: DL-002, DL-003, DL-004)
- **CI-M-003-006** `crates/game_app/src/plugins/app_shell.rs`: Leave UI spawning and state transitions in AppShellPlugin while delegating copy and decision branches to extracted helpers that stay in coverage scope. (refs: DL-002, DL-003, DL-004)
- **CI-M-003-007** `crates/game_app/src/plugins/mod.rs`: Wire extracted app_shell_logic and save_load_logic modules as `pub(crate)` through plugin exports and add `MenuContext` to the public re-exports so integration tests use real types. (refs: DL-003, DL-004)
- **CI-M-003-007b** `crates/game_app/src/lib.rs`: Add `MenuContext` to the public re-exports and a `#[doc(hidden)] test_support` module that re-exports the extracted logic modules for integration tests, replacing the `#[path]`/shim pattern. (refs: DL-003, DL-004)
- **CI-M-003-008** `crates/game_app/tests/local_match_flow.rs`: Keep the local match integration flow covering launch-to-result behavior after logic extraction so shell refactors do not silently break core play orchestration. (refs: DL-003, DL-004)
- **CI-M-003-009** `crates/game_app/tests/match_state_flow.rs`: Expand recovery and snapshot integration assertions around MatchSession restore and dirty-state transitions so the shell bridge still matches the extracted logic expectations. (refs: DL-003, DL-004)
- **CI-M-003-010** `crates/game_app/tests/save_load_flow.rs`: Cover manual save, load, resume, clear, and autosave recovery flows end to end so repository wiring stays honest after save_load logic extraction. (refs: DL-002, DL-003, DL-004)
- **CI-M-003-011** `crates/game_app/tests/promotion_flow.rs`: Preserve promotion and overlay integration coverage so targeted input-system tests do not replace the one full flow that proves staged promotion behavior in the shell. (refs: DL-003, DL-004)
- **CI-M-003-012** `crates/game_app/tests/app_shell_logic.rs`: Add deterministic unit coverage for extracted app-shell helper branches including copy selection, result labels, confirmation text, and recovery policy presentation. Uses real `game_app` types via `test_support` re-exports instead of shim redefinitions. (refs: DL-003, DL-004)
- **CI-M-003-013** `crates/game_app/tests/save_load_logic.rs`: Add deterministic unit coverage for extracted save-load helper branches including banner visibility, error aggregation, slot labeling, and threshold messaging. Uses real `game_app` types via `test_support` re-exports instead of `#[path]` includes with module-path shims. (refs: DL-002, DL-003, DL-004, DL-005)

#### Code Changes

**CC-M-003-001** (crates/game_app/src/match_state.rs) - implements CI-M-003-001

**Code:**

```diff
--- a/crates/game_app/src/match_state.rs
+++ b/crates/game_app/src/match_state.rs
@@ -231,4 +231,65 @@ impl Default for MatchSession {
     fn default() -> Self {
         Self::start_local_match()
     }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn square(name: &str) -> Square {
+        Square::from_algebraic(name).expect("test square must be valid")
+    }
+
+    fn sample_snapshot(dirty_recovery: bool) -> GameSnapshot {
+        GameSnapshot::from_parts(
+            GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("fixture FEN should parse"),
+            SnapshotMetadata {
+                label: String::from("Fixture"),
+                created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
+                updated_at_utc: None,
+                notes: None,
+                save_kind: chess_persistence::SaveKind::Manual,
+                session_id: String::from("fixture"),
+                recovery_key: None,
+            },
+            SnapshotShellState {
+                selected_square: Some(square("e7")),
+                pending_promotion: Some(PendingPromotionSnapshot { from: square("e7"), to: square("e8") }),
+                last_move: Some(Move::new(square("e7"), square("e8"))),
+                claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
+                dirty_recovery,
+            },
+        )
+    }
+
+    #[test]
+    fn restore_from_snapshot_keeps_shell_bridge_fields() {
+        let session = MatchSession::restore_from_snapshot(&sample_snapshot(true));
+        assert_eq!(session.selected_square, Some(square("e7")));
+        assert_eq!(session.pending_promotion_move, Some(Move::new(square("e7"), square("e8"))));
+        assert_eq!(session.claimed_draw_reason(), Some(ClaimedDrawReason::ThreefoldRepetition));
+        assert!(session.summary().pending_promotion);
+        assert!(session.summary().dirty_recovery);
+    }
+
+    #[test]
+    fn replace_game_state_clears_interaction_and_marks_recovery_dirty() {
+        let mut session = MatchSession::restore_from_snapshot(&sample_snapshot(false));
+        session.replace_game_state(GameState::starting_position());
+
+        assert_eq!(session.selected_square, None);
+        assert_eq!(session.pending_promotion_move, None);
+        assert_eq!(session.last_move, None);
+        assert!(session.is_recovery_dirty());
+    }
+
+    #[test]
+    fn claim_draw_updates_summary_without_reaching_through_game_state() {
+        let mut session = MatchSession::start_local_match();
+        session.replace_game_state(GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1").expect("fixture FEN should parse"));
+        assert!(session.claim_draw());
+        assert!(session.is_finished());
+        assert!(session.summary().dirty_recovery);
+    }
 }

```

**Documentation:**

```diff
--- a/crates/game_app/src/match_state.rs
+++ b/crates/game_app/src/match_state.rs
@@ -1,5 +1,6 @@
 //! Bevy-facing match bridge for local play, load, and recovery flows.
 //! Snapshot conversion keeps `chess_core` authoritative while the shell restores only the interaction state it needs. (ref: DL-001) (ref: DL-004)
+//! MatchSession tests pin shell-bridge summary fields directly so recovery dirtiness and draw-claim branches stay measurable outside Bevy UI flows. (ref: DL-004) (ref: DL-007)
 
 use bevy::prelude::Resource;
 use chess_core::{DrawAvailability, GameState, GameStatus, Move, MoveError, Piece, Square};

```


**CC-M-003-002** (crates/game_app/src/plugins/input.rs) - implements CI-M-003-002

**Code:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -208,6 +208,14 @@
     use bevy::ecs::system::SystemState;
     use chess_core::Square;
 
+    type KeyboardActionSystemState<'w, 's> = SystemState<(
+        Option<Res<'w, ButtonInput<KeyCode>>>,
+        Res<'w, ShellMenuState>,
+        ResMut<'w, MatchSession>,
+        MessageWriter<'w, MenuAction>,
+        MessageWriter<'w, SaveLoadRequest>,
+    )>;
+
     type SquareClickSystemState<'w, 's> = SystemState<(
         Option<Res<'w, ButtonInput<MouseButton>>>,
         Res<'w, HoveredSquare>,
@@ -250,4 +258,64 @@
         }));
         assert!(!overlay_captures_match_input(&ShellMenuState::default()));
     }
-}
+
+#[test]
+fn clicking_selected_square_deselects_and_marks_recovery_dirty() {
+    let mut world = World::new();
+    let mut mouse_buttons = ButtonInput::<MouseButton>::default();
+    mouse_buttons.press(MouseButton::Left);
+    world.insert_resource(mouse_buttons);
+    world.insert_resource(HoveredSquare(Some(
+        Square::from_algebraic("e2").expect("valid square"),
+    )));
+    world.insert_resource(ShellMenuState::default());
+
+    let mut match_session = MatchSession::start_local_match();
+    match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
+    match_session.mark_recovery_persisted();
+    world.insert_resource(match_session);
+
+    let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
+    let (mouse_buttons, hovered_square, menu_state, match_session) =
+        system_state.get_mut(&mut world);
+    handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);
+
+    let match_session = world.resource::<MatchSession>();
+    assert_eq!(match_session.selected_square, None);
+    assert!(match_session.is_recovery_dirty());
+}
+
+#[test]
+fn escape_clears_pending_promotion_before_pause_overlay() {
+    let mut app = App::new();
+    app.add_message::<MenuAction>();
+    app.add_message::<SaveLoadRequest>();
+    app.insert_resource(ButtonInput::<KeyCode>::default());
+    app.insert_resource(ShellMenuState::default());
+
+    let mut match_session = MatchSession::start_local_match();
+    match_session.pending_promotion_move = Some(Move::new(
+        Square::from_algebraic("e7").expect("valid square"),
+        Square::from_algebraic("e8").expect("valid square"),
+    ));
+    app.insert_resource(match_session);
+    app.world_mut()
+        .resource_mut::<ButtonInput<KeyCode>>()
+        .press(KeyCode::Escape);
+
+    let mut system_state: KeyboardActionSystemState<'_, '_> = SystemState::new(app.world_mut());
+    let (keyboard_input, menu_state, match_session, menu_actions, save_requests) =
+        system_state.get_mut(app.world_mut());
+    handle_keyboard_match_actions(
+        keyboard_input,
+        menu_state,
+        match_session,
+        menu_actions,
+        save_requests,
+    );
+
+    let match_session = app.world().resource::<MatchSession>();
+    assert_eq!(match_session.pending_promotion_move, None);
+    assert!(match_session.is_recovery_dirty());
+}
+}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -1,3 +1,4 @@
+//! Input coverage stays focused on selection, promotion, and overlay guards so this hotspot improves through deterministic systems tests instead of broader scene smoke. (ref: DL-004) (ref: DL-007)
 use bevy::prelude::*;
 use bevy::window::PrimaryWindow;
 use chess_core::{Move, PieceKind};

```


**CC-M-003-003** (crates/game_app/src/plugins/save_load_logic.rs) - implements CI-M-003-003

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/save_load_logic.rs
@@ -0,0 +1,65 @@
+use chess_persistence::{RecoveryStartupPolicy, SavedSessionSummary};
+
+use crate::plugins::menu::RecoveryBannerState;
+use crate::plugins::save_load::SaveLoadState;
+
+pub fn combine_persistence_errors(errors: impl IntoIterator<Item = Option<String>>) -> Option<String> {
+    let messages = errors.into_iter().flatten().collect::<Vec<_>>();
+    if messages.is_empty() {
+        None
+    } else {
+        Some(messages.join(" "))
+    }
+}
+
+pub fn manual_save_message(summary: &SavedSessionSummary) -> String {
+    format!("Saved match as {}.", summary.label)
+}
+
+pub fn deleted_save_message(slot_id: &str) -> String {
+    format!("Deleted save {slot_id}.")
+}
+
+pub fn recovery_banner_label(recovery: Option<&SavedSessionSummary>) -> Option<String> {
+    recovery.map(|summary| summary.label.clone())
+}
+
+pub fn hide_recovery_banner(recovery_banner: &mut RecoveryBannerState) {
+    recovery_banner.available = false;
+    recovery_banner.dirty = false;
+    recovery_banner.label = None;
+}
+
+pub fn sync_cached_recovery_visibility(
+    save_state: &SaveLoadState,
+    recovery_banner: &mut RecoveryBannerState,
+) {
+    let Some(summary) = save_state.recovery.as_ref() else {
+        hide_recovery_banner(recovery_banner);
+        return;
+    };
+
+    if save_state.settings.recovery_policy == RecoveryStartupPolicy::Ignore {
+        hide_recovery_banner(recovery_banner);
+        return;
+    }
+
+    recovery_banner.available = true;
+    recovery_banner.dirty = false;
+    recovery_banner.label = recovery_banner_label(Some(summary));
+}
+
+pub fn recovery_policy_status_copy(policy: RecoveryStartupPolicy) -> &'static str {
+    match policy {
+        RecoveryStartupPolicy::Resume => {
+            "Resume automatically routes the stored interrupted session through MatchLoading."
+        }
+        RecoveryStartupPolicy::Ask => {
+            "Ask keeps interrupted-session recovery visible without forcing a startup route."
+        }
+        RecoveryStartupPolicy::Ignore => {
+            "Ignore hides interrupted-session affordances without deleting the stored snapshot."
+        }
+    }
+}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/save_load_logic.rs
+++ b/crates/game_app/src/plugins/save_load_logic.rs
@@ -1,3 +1,4 @@
+//! Extracted save/load helpers keep banner and message decisions in pure functions so coverage gains come from direct branch tests without excluding the orchestration layer. (ref: DL-002) (ref: DL-004) (ref: DL-007)
 use chess_persistence::{RecoveryStartupPolicy, SavedSessionSummary};
 
 use crate::plugins::menu::RecoveryBannerState;

```


**CC-M-003-004** (crates/game_app/src/plugins/save_load.rs) - implements CI-M-003-004

**Code:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -11,6 +11,7 @@
 };
 
 use super::menu::{MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState};
+use super::save_load_logic;
 use crate::app::AppScreenState;
 use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
 
@@ -102,7 +103,7 @@
     };
     let store_resource = SessionStoreResource(store.clone());
     // Startup preloads the save index and recovery banner from the repository so the main menu reflects persisted shell state immediately. (ref: DL-003) (ref: DL-008)
-    save_state.last_error = combine_persistence_errors([
+    save_state.last_error = save_load_logic::combine_persistence_errors([
         startup_error,
         refresh_store_index_from_resource(&store_resource, &mut save_state, &mut recovery_banner),
     ]);
@@ -132,7 +133,7 @@
             }
         }
         RecoveryStartupPolicy::Ignore => {
-            sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+            save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
         }
         RecoveryStartupPolicy::Ask => {}
     }
@@ -195,7 +196,7 @@
                     Ok(summary) => {
                         save_state.last_error = None;
                         save_state.last_message =
-                            Some(format!("Saved match as {}.", summary.label));
+                            Some(save_load_logic::manual_save_message(&summary));
                         menu_state.selected_save = Some(summary.slot_id.clone());
                         save_state.last_error = refresh_store_index_from_resource(
                             &store,
@@ -224,7 +225,7 @@
             SaveLoadRequest::DeleteManual { slot_id } => match store.0.delete_manual(slot_id) {
                 Ok(()) => {
                     save_state.last_error = None;
-                    save_state.last_message = Some(format!("Deleted save {slot_id}."));
+                    save_state.last_message = Some(save_load_logic::deleted_save_message(slot_id));
                     if menu_state.selected_save.as_deref() == Some(slot_id.as_str()) {
                         menu_state.selected_save = None;
                     }
@@ -286,11 +287,11 @@
                 Ok(()) => {
                     save_state.last_error = None;
                     save_state.last_message = Some(String::from("Saved shell settings."));
-                    sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+                    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
                 }
                 Err(_) => {
                     save_state.last_error = Some(String::from("Unable to save shell settings."));
-                    sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+                    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
                 }
             },
         }
@@ -352,7 +353,7 @@
             save_state.last_error = Some(String::from(
                 "Unable to clear interrupted-session recovery.",
             ));
-            sync_cached_recovery_visibility(save_state, recovery_banner);
+            save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
         }
     }
 }
@@ -392,7 +393,7 @@
         )),
     }
 
-    combine_persistence_errors(errors.into_iter().map(Some))
+    save_load_logic::combine_persistence_errors(errors.into_iter().map(Some))
 }
 
 fn resolve_session_store(
@@ -417,23 +418,13 @@
         }
     }
 }
-
-fn combine_persistence_errors(errors: impl IntoIterator<Item = Option<String>>) -> Option<String> {
-    let messages = errors.into_iter().flatten().collect::<Vec<_>>();
-    if messages.is_empty() {
-        None
-    } else {
-        Some(messages.join(" "))
-    }
-}
-
 fn set_cached_recovery(
     recovery: Option<SavedSessionSummary>,
     save_state: &mut SaveLoadState,
     recovery_banner: &mut RecoveryBannerState,
 ) {
     save_state.recovery = recovery;
-    sync_cached_recovery_visibility(save_state, recovery_banner);
+    save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
 }
 
 fn clear_cached_recovery(
@@ -441,34 +432,8 @@
     recovery_banner: &mut RecoveryBannerState,
 ) {
     save_state.recovery = None;
-    hide_recovery_banner(recovery_banner);
-}
-
-fn hide_recovery_banner(recovery_banner: &mut RecoveryBannerState) {
-    recovery_banner.available = false;
-    recovery_banner.dirty = false;
-    recovery_banner.label = None;
-}
-
-fn sync_cached_recovery_visibility(
-    save_state: &SaveLoadState,
-    recovery_banner: &mut RecoveryBannerState,
-) {
-    let Some(summary) = save_state.recovery.as_ref() else {
-        hide_recovery_banner(recovery_banner);
-        return;
-    };
-
-    if save_state.settings.recovery_policy == RecoveryStartupPolicy::Ignore {
-        hide_recovery_banner(recovery_banner);
-        return;
-    }
-
-    recovery_banner.available = true;
-    recovery_banner.dirty = false;
-    recovery_banner.label = Some(summary.label.clone());
-}
-
+    save_load_logic::hide_recovery_banner(recovery_banner);
+}
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -584,11 +549,11 @@
         };
         let mut recovery_banner = RecoveryBannerState::default();
 
-        sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
         assert!(!recovery_banner.available);
 
         save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
-        sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
 
         assert!(recovery_banner.available);
         assert_eq!(recovery_banner.label.as_deref(), Some("Recovery Fixture"));
@@ -613,7 +578,7 @@
         };
         let mut recovery_banner = RecoveryBannerState::default();
 
-        sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
+        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
         clear_result_recovery_cache(&store, &mut save_state, &mut recovery_banner);
 
         assert_eq!(

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -1,5 +1,6 @@
 //! Shell persistence orchestration for manual saves, interrupted-session recovery, and settings.
 //! Repository I/O lives here so manual saves, interrupted-session recovery, and the shipped settings trio of startup recovery, destructive confirmations, and display mode stay behind one snapshot-based boundary. (ref: DL-002) (ref: DL-005) (ref: DL-007)
+//! Extracted helpers carry branch-heavy copy and recovery-visibility rules so the Bevy plugin remains in scope while direct tests cover the decision surface. (ref: DL-002) (ref: DL-004) (ref: DL-007)
 
 use std::path::PathBuf;
 

```


**CC-M-003-005** (crates/game_app/src/plugins/app_shell_logic.rs) - implements CI-M-003-005

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/app_shell_logic.rs
@@ -0,0 +1,160 @@
+use chess_core::{
+    AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Move, Side, WinReason,
+};
+use chess_persistence::{DisplayMode, RecoveryStartupPolicy, SavedSessionSummary};
+
+use crate::app::AppScreenState;
+use crate::match_state::ClaimedDrawReason;
+use crate::plugins::menu::{ConfirmationKind, MenuContext, RecoveryBannerState, ShellMenuState};
+use crate::plugins::save_load::SaveLoadState;
+
+pub fn return_to_menu_abandons_active_match(
+    state: AppScreenState,
+    menu_state: &ShellMenuState,
+) -> bool {
+    state == AppScreenState::InMatch && menu_state.context == MenuContext::InMatchOverlay
+}
+
+pub fn effective_shell_status(
+    menu_state: &ShellMenuState,
+    save_state: &SaveLoadState,
+    recovery: &RecoveryBannerState,
+) -> Option<String> {
+    save_state
+        .last_error
+        .clone()
+        .or_else(|| save_state.last_message.clone())
+        .or_else(|| menu_state.status_line.clone())
+        .or_else(|| {
+            if recovery.available {
+                recovery.label.as_ref().map(|label| {
+                    format!("Interrupted-session recovery is available as {label}.")
+                })
+            } else {
+                None
+            }
+        })
+}
+
+pub fn derive_save_label(last_move: Option<Move>) -> String {
+    if let Some(last_move) = last_move {
+        format!("Local Match after {last_move}")
+    } else {
+        String::from("Local Match Save")
+    }
+}
+
+pub fn selected_save_summary<'a>(
+    menu_state: &ShellMenuState,
+    save_state: &'a SaveLoadState,
+) -> Option<&'a SavedSessionSummary> {
+    let slot_id = menu_state.selected_save.as_deref()?;
+    save_state
+        .manual_saves
+        .iter()
+        .find(|summary| summary.slot_id == slot_id)
+}
+
+pub fn next_recovery_policy(current: RecoveryStartupPolicy) -> RecoveryStartupPolicy {
+    match current {
+        RecoveryStartupPolicy::Resume => RecoveryStartupPolicy::Ask,
+        RecoveryStartupPolicy::Ask => RecoveryStartupPolicy::Ignore,
+        RecoveryStartupPolicy::Ignore => RecoveryStartupPolicy::Resume,
+    }
+}
+
+pub fn recovery_policy_label(policy: RecoveryStartupPolicy) -> &'static str {
+    match policy {
+        RecoveryStartupPolicy::Resume => "Resume automatically",
+        RecoveryStartupPolicy::Ask => "Ask on startup",
+        RecoveryStartupPolicy::Ignore => "Ignore recovery on startup",
+    }
+}
+
+pub fn display_mode_label(mode: DisplayMode) -> &'static str {
+    match mode {
+        DisplayMode::Windowed => "Windowed",
+        DisplayMode::Fullscreen => "Fullscreen",
+    }
+}
+
+pub fn toggle_label(label: &str, enabled: bool) -> String {
+    if enabled {
+        format!("{label}: on")
+    } else {
+        format!("{label}: off")
+    }
+}
+
+pub fn confirmation_copy(kind: ConfirmationKind) -> (&'static str, &'static str) {
+    match kind {
+        ConfirmationKind::AbandonMatch => (
+            "Leave the current match?",
+            "Clearing the recovery slot prevents startup resume from restoring this position.",
+        ),
+        ConfirmationKind::DeleteSave => (
+            "Delete the selected save?",
+            "Manual save history is user-controlled so deletes stay explicit.",
+        ),
+        ConfirmationKind::OverwriteSave => (
+            "Overwrite the selected save?",
+            "Manual saves stay distinct from recovery, so overwrites should always be deliberate.",
+        ),
+    }
+}
+
+pub fn match_session_result_title(status: GameStatus, claimed_draw: Option<ClaimedDrawReason>) -> String {
+    if let Some(claimed_draw_reason) = claimed_draw {
+        return match claimed_draw_reason {
+            ClaimedDrawReason::ThreefoldRepetition => String::from("Draw Claimed by Repetition"),
+            ClaimedDrawReason::FiftyMoveRule => {
+                String::from("Draw Claimed by Fifty-Move Rule")
+            }
+        };
+    }
+
+    match status {
+        GameStatus::Ongoing { .. } => String::from("Match Complete"),
+        GameStatus::Finished(GameOutcome::Win {
+            winner: Side::White,
+            reason: WinReason::Checkmate,
+        }) => String::from("White Wins"),
+        GameStatus::Finished(GameOutcome::Win {
+            winner: Side::Black,
+            reason: WinReason::Checkmate,
+        }) => String::from("Black Wins"),
+        GameStatus::Finished(GameOutcome::Draw(_)) => String::from("Draw"),
+    }
+}
+
+pub fn match_session_result_detail(status: GameStatus, claimed_draw: Option<ClaimedDrawReason>) -> String {
+    if let Some(claimed_draw_reason) = claimed_draw {
+        return match claimed_draw_reason {
+            ClaimedDrawReason::ThreefoldRepetition => {
+                String::from("Threefold repetition was claimed from the in-match HUD.")
+            }
+            ClaimedDrawReason::FiftyMoveRule => {
+                String::from("The fifty-move rule was claimed from the in-match HUD.")
+            }
+        };
+    }
+
+    match status {
+        GameStatus::Ongoing { .. } => {
+            String::from("The shell routes to results only after chess_core resolves the outcome.")
+        }
+        GameStatus::Finished(GameOutcome::Win {
+            reason: WinReason::Checkmate,
+            ..
+        }) => String::from("Checkmate detected by chess_core."),
+        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {
+            String::from("Stalemate detected by chess_core.")
+        }
+        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
+            AutomaticDrawReason::FivefoldRepetition,
+        ))) => String::from("Fivefold repetition detected by chess_core."),
+        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
+            AutomaticDrawReason::SeventyFiveMoveRule,
+        ))) => String::from("Seventy-five move rule detected by chess_core."),
+    }
+}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell_logic.rs
+++ b/crates/game_app/src/plugins/app_shell_logic.rs
@@ -1,3 +1,4 @@
+//! These helpers hold branchy shell copy and selection rules so `game_app` coverage grows through direct logic tests rather than denominator trimming. (ref: DL-002) (ref: DL-004) (ref: DL-007)
 use chess_core::{
     AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Side, WinReason,
 };

```


**CC-M-003-006** (crates/game_app/src/plugins/app_shell.rs) - implements CI-M-003-006

**Code:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@ -2,17 +2,16 @@
 //! Main menu, pause overlay, and results render from modal resources while match launch still funnels through MatchLoading. (ref: DL-001) (ref: DL-007)
 
 use bevy::prelude::*;
-use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, PieceKind, WinReason};
+use chess_core::PieceKind;
 use chess_persistence::{DisplayMode, RecoveryStartupPolicy, SavedSessionSummary};
 
+use super::app_shell_logic;
 use super::menu::{
     ConfirmationKind, MenuAction, MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState,
 };
 use super::save_load::{SaveLoadRequest, SaveLoadState};
 use crate::app::AppScreenState;
-use crate::match_state::{
-    ClaimedDrawReason, MatchLaunchIntent, MatchSession, PendingLoadedSnapshot,
-};
+use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
 use crate::style::ShellTheme;
 
 pub struct AppShellPlugin;
@@ -1092,7 +1091,7 @@
     state: AppScreenState,
     menu_state: &ShellMenuState,
 ) -> bool {
-    state == AppScreenState::InMatch && menu_state.context == MenuContext::InMatchOverlay
+    app_shell_logic::return_to_menu_abandons_active_match(state, menu_state)
 }
 
 fn advance_to_match_result(
@@ -1170,142 +1169,47 @@
     save_state: &SaveLoadState,
     recovery: &RecoveryBannerState,
 ) -> Option<String> {
-    save_state
-        .last_error
-        .clone()
-        .or_else(|| save_state.last_message.clone())
-        .or_else(|| menu_state.status_line.clone())
-        .or_else(|| {
-            if recovery.available {
-                recovery
-                    .label
-                    .as_ref()
-                    .map(|label| format!("Interrupted-session recovery is available as {label}."))
-            } else {
-                None
-            }
-        })
+    app_shell_logic::effective_shell_status(menu_state, save_state, recovery)
 }
 
 fn derive_save_label(match_session: &MatchSession) -> String {
-    if let Some(last_move) = match_session.last_move {
-        format!("Local Match after {last_move}")
-    } else {
-        String::from("Local Match Save")
-    }
+    app_shell_logic::derive_save_label(match_session.last_move)
 }
 
 fn selected_save_summary<'a>(
     menu_state: &ShellMenuState,
     save_state: &'a SaveLoadState,
 ) -> Option<&'a SavedSessionSummary> {
-    let slot_id = menu_state.selected_save.as_deref()?;
-    save_state
-        .manual_saves
-        .iter()
-        .find(|summary| summary.slot_id == slot_id)
+    app_shell_logic::selected_save_summary(menu_state, save_state)
 }
 
 fn next_recovery_policy(current: RecoveryStartupPolicy) -> RecoveryStartupPolicy {
-    match current {
-        RecoveryStartupPolicy::Resume => RecoveryStartupPolicy::Ask,
-        RecoveryStartupPolicy::Ask => RecoveryStartupPolicy::Ignore,
-        RecoveryStartupPolicy::Ignore => RecoveryStartupPolicy::Resume,
-    }
+    app_shell_logic::next_recovery_policy(current)
 }
 
 fn recovery_policy_label(policy: RecoveryStartupPolicy) -> &'static str {
-    match policy {
-        RecoveryStartupPolicy::Resume => "Resume automatically",
-        RecoveryStartupPolicy::Ask => "Ask on startup",
-        RecoveryStartupPolicy::Ignore => "Ignore recovery on startup",
-    }
+    app_shell_logic::recovery_policy_label(policy)
 }
 
 fn display_mode_label(mode: DisplayMode) -> &'static str {
-    match mode {
-        DisplayMode::Windowed => "Windowed",
-        DisplayMode::Fullscreen => "Fullscreen",
-    }
+    app_shell_logic::display_mode_label(mode)
 }
 
 fn toggle_label(label: &str, enabled: bool) -> String {
-    if enabled {
-        format!("{label}: on")
-    } else {
-        format!("{label}: off")
-    }
+    app_shell_logic::toggle_label(label, enabled)
 }
 
 /// Supplies confirmation copy for the destructive-confirmation slice of the shipped shell settings contract. (ref: DL-005)
 fn confirmation_copy(kind: ConfirmationKind) -> (&'static str, &'static str) {
-    match kind {
-        ConfirmationKind::AbandonMatch => (
-            "Leave the current match?",
-            "Clearing the recovery slot prevents startup resume from restoring this position.",
-        ),
-        ConfirmationKind::DeleteSave => (
-            "Delete the selected save?",
-            "Manual save history is user-controlled so deletes stay explicit.",
-        ),
-        ConfirmationKind::OverwriteSave => (
-            "Overwrite the selected save?",
-            "Manual saves stay distinct from recovery, so overwrites should always be deliberate.",
-        ),
-    }
+    app_shell_logic::confirmation_copy(kind)
 }
 
 fn match_session_result_title(match_session: &MatchSession) -> String {
-    if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
-        return match claimed_draw_reason {
-            ClaimedDrawReason::ThreefoldRepetition => String::from("Draw Claimed by Repetition"),
-            ClaimedDrawReason::FiftyMoveRule => String::from("Draw Claimed by Fifty-Move Rule"),
-        };
-    }
-
-    match match_session.status() {
-        chess_core::GameStatus::Ongoing { .. } => String::from("Match Complete"),
-        chess_core::GameStatus::Finished(GameOutcome::Win {
-            winner,
-            reason: WinReason::Checkmate,
-        }) => match winner {
-            chess_core::Side::White => String::from("White Wins"),
-            chess_core::Side::Black => String::from("Black Wins"),
-        },
-        chess_core::GameStatus::Finished(GameOutcome::Draw(_)) => String::from("Draw"),
-    }
+    app_shell_logic::match_session_result_title(match_session.status(), match_session.claimed_draw_reason())
 }
 
 fn match_session_result_detail(match_session: &MatchSession) -> String {
-    if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
-        return match claimed_draw_reason {
-            ClaimedDrawReason::ThreefoldRepetition => {
-                String::from("Threefold repetition was claimed from the in-match HUD.")
-            }
-            ClaimedDrawReason::FiftyMoveRule => {
-                String::from("The fifty-move rule was claimed from the in-match HUD.")
-            }
-        };
-    }
-
-    match match_session.status() {
-        chess_core::GameStatus::Ongoing { .. } => {
-            String::from("The shell routes to results only after chess_core resolves the outcome.")
-        }
-        chess_core::GameStatus::Finished(GameOutcome::Win {
-            reason: WinReason::Checkmate,
-            ..
-        }) => String::from("Checkmate detected by chess_core."),
-        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {
-            String::from("Stalemate detected by chess_core.")
-        }
-        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
-            AutomaticDrawReason::FivefoldRepetition,
-        ))) => String::from("Fivefold repetition detected by chess_core."),
-        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
-            AutomaticDrawReason::SeventyFiveMoveRule,
-        ))) => String::from("Seventy-five move rule detected by chess_core."),
-    }
+    app_shell_logic::match_session_result_detail(match_session.status(), match_session.claimed_draw_reason())
 }
 
 #[cfg(test)]

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@ -1,5 +1,6 @@
 //! Presentation layer for the coarse app shell.
 //! Main menu, pause overlay, and results render from modal resources while match launch still funnels through MatchLoading. (ref: DL-001) (ref: DL-007)
+//! The orchestration layer stays in coverage scope while extracted helpers absorb copy and routing branches into deterministic tests. (ref: DL-002) (ref: DL-004) (ref: DL-007)
 
 use bevy::prelude::*;
 use chess_core::PieceKind;

```


**CC-M-003-007** (crates/game_app/src/plugins/mod.rs) - implements CI-M-003-007

**Code:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,10 +1,12 @@
 mod app_shell;
+pub(crate) mod app_shell_logic;
 mod board_scene;
 mod input;
 mod menu;
 mod move_feedback;
 mod piece_view;
 mod save_load;
+pub(crate) mod save_load_logic;
 mod scaffold;

 pub use app_shell::AppShellPlugin;
+pub use menu::MenuContext;

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,3 +1,4 @@
+//! Plugin exports keep extracted logic modules explicit in the shell graph so branch coverage stays visible instead of hiding behind broad integration tests. (ref: DL-004)
 mod app_shell;
 mod app_shell_logic;
 mod board_scene;

```


**CC-M-003-007b** (crates/game_app/src/lib.rs) - implements CI-M-003-007

**Code:**

```diff
--- a/crates/game_app/src/lib.rs
+++ b/crates/game_app/src/lib.rs
@@ -12,7 +12,16 @@ pub use match_state::{
 pub use plugins::{
     AiMatchPlugin, AppShellPlugin, BoardScenePlugin, BoardSquareVisual, ChessAudioPlugin,
     ConfirmationKind, MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin, PieceViewPlugin,
-    PieceVisual, RecoveryBannerState, SaveLoadPlugin, SaveLoadRequest, SaveLoadState,
+    MenuContext, PieceVisual, RecoveryBannerState, SaveLoadPlugin, SaveLoadRequest, SaveLoadState,
     SaveRootOverride, SessionStoreResource, ShellInputPlugin, ShellMenuState,
 };
 pub use style::ShellTheme;
+
+/// Re-exports for integration tests that need direct access to extracted logic modules.
+#[doc(hidden)]
+pub mod test_support {
+    pub use crate::plugins::app_shell_logic;
+    pub use crate::plugins::save_load_logic;
+}

```


**CC-M-003-008** (crates/game_app/tests/local_match_flow.rs) - implements CI-M-003-008

**Code:**

```diff
--- a/crates/game_app/tests/local_match_flow.rs
+++ b/crates/game_app/tests/local_match_flow.rs
@@ -100,4 +100,13 @@ fn local_match_flow_covers_start_move_claim_draw_and_result_transition() {
     app.update();

     assert_eq!(current_state(&app), AppScreenState::MatchResult);
+
+    app.world_mut().write_message(MenuAction::Rematch);
+    app.update();
+    app.update();
+    app.update();
+
+    assert_eq!(current_state(&app), AppScreenState::InMatch);
+    assert_eq!(app.world().resource::<MatchSession>().game_state().to_fen(), GameState::starting_position().to_fen());
+    assert!(app.world().resource::<MatchSession>().summary().dirty_recovery);
 }

```

**Documentation:**

```diff
--- a/crates/game_app/tests/local_match_flow.rs
+++ b/crates/game_app/tests/local_match_flow.rs
@@ -1,3 +1,4 @@
+//! This flow keeps one rematch end-to-end anchor in place while `game_app` coverage stays concentrated in extracted logic tests. (ref: DL-003) (ref: DL-004)
 use tempfile::tempdir;
 
 use bevy::prelude::*;

```


**CC-M-003-009** (crates/game_app/tests/match_state_flow.rs) - implements CI-M-003-009

**Code:**

```diff
--- a/crates/game_app/tests/match_state_flow.rs
+++ b/crates/game_app/tests/match_state_flow.rs
@@ -119,8 +119,12 @@ fn manual_load_intent_restores_snapshot_and_enters_in_match() {
     assert_eq!(
         match_session.pending_promotion_move,
         Some(Move::new(
             Square::from_algebraic("e7").expect("valid square"),
             Square::from_algebraic("e8").expect("valid square"),
         ))
     );
+    assert_eq!(match_session.selected_square, Some(Square::from_algebraic("e7").expect("valid square")));
+    assert_eq!(match_session.last_move, Some(Move::new(Square::from_algebraic("e7").expect("valid square"), Square::from_algebraic("e8").expect("valid square"))));
+    assert!(match_session.summary().pending_promotion);
+    assert!(match_session.is_recovery_dirty());
 }
@@ -177,5 +181,16 @@ fn startup_resume_policy_hydrates_recovery_snapshot_through_match_loading() {
     assert_eq!(
         app.world().resource::<MatchSession>().selected_square,
         Some(Square::from_algebraic("e7").expect("valid square"))
     );
+}
+
+#[test]
+fn restored_snapshot_summary_matches_shell_bridge_fields() {
+    let snapshot = sample_snapshot("Summary Fixture");
+    let match_session = MatchSession::restore_from_snapshot(&snapshot);
+    let summary = match_session.summary();
+
+    assert_eq!(summary.last_move, Some(Move::new(Square::from_algebraic("e7").expect("valid square"), Square::from_algebraic("e8").expect("valid square"))));
+    assert!(summary.pending_promotion);
+    assert!(summary.dirty_recovery);
 }

```

**Documentation:**

```diff
--- a/crates/game_app/tests/match_state_flow.rs
+++ b/crates/game_app/tests/match_state_flow.rs
@@ -1,3 +1,4 @@
+//! Snapshot-summary checks keep shell-bridge fields aligned with persistence restores, which prevents `game_app` coverage from drifting away from repository behavior. (ref: DL-004) (ref: DL-007)
 use chess_persistence::{
     GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore,
     ShellSettings, SnapshotMetadata, SnapshotShellState,

```


**CC-M-003-010** (crates/game_app/tests/save_load_flow.rs) - implements CI-M-003-010

**Code:**

```diff
--- a/crates/game_app/tests/save_load_flow.rs
+++ b/crates/game_app/tests/save_load_flow.rs
@@ -334,7 +334,47 @@ fn entering_match_result_clears_recovery_label_cache() {
     assert!(
         store
             .load_recovery()
             .expect("recovery load should succeed")
             .is_none()
     );
+}
+
+#[test]
+fn delete_manual_save_clears_selected_slot_and_refreshes_index() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
+    enter_local_match(&mut app);
+
+    app.world_mut().write_message(SaveLoadRequest::SaveManual {
+        label: String::from("Delete Me"),
+        slot_id: None,
+    });
+    app.update();
+    app.update();
+
+    let slot_id = app.world().resource::<SaveLoadState>().manual_saves[0].slot_id.clone();
+    app.world_mut().resource_mut::<ShellMenuState>().selected_save = Some(slot_id.clone());
+    app.world_mut().write_message(SaveLoadRequest::DeleteManual { slot_id });
+    app.update();
+
+    assert!(app.world().resource::<SaveLoadState>().manual_saves.is_empty());
+    assert_eq!(app.world().resource::<ShellMenuState>().selected_save, None);
+}
+
+#[test]
+fn clear_recovery_request_hides_banner_without_touching_manual_save_index() {
+    let root = tempdir().expect("temporary directory should be created");
+    let store = SessionStore::new(root.path());
+    store.store_recovery(recovery_snapshot("Recovery Fixture")).expect("recovery fixture should be written");
+
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
+    app.world_mut().write_message(SaveLoadRequest::ClearRecovery);
+    app.update();
+
+    let recovery = app.world().resource::<RecoveryBannerState>();
+    assert!(!recovery.available);
+    assert_eq!(recovery.label, None);
+    assert!(store.load_recovery().expect("recovery load should succeed").is_none());
 }

```

**Documentation:**

```diff
--- a/crates/game_app/tests/save_load_flow.rs
+++ b/crates/game_app/tests/save_load_flow.rs
@@ -1,3 +1,4 @@
+//! These flows keep real repository boundaries covered while helper extraction absorbs the cheaper branch matrix. (ref: DL-003) (ref: DL-004)
 use std::fs;
 
 use chess_persistence::{

```


**CC-M-003-011** (crates/game_app/tests/promotion_flow.rs) - implements CI-M-003-011

**Code:**

```diff
--- a/crates/game_app/tests/promotion_flow.rs
+++ b/crates/game_app/tests/promotion_flow.rs
@@ -39,6 +39,10 @@ fn enter_local_match(app: &mut App) {
     app.update();
 }

+fn current_state(app: &App) -> AppScreenState {
+    *app.world().resource::<State<AppScreenState>>().get()
+}
+
 fn ui_texts(app: &mut App) -> Vec<String> {
     let world = app.world_mut();
     let mut query = world.query::<&Text>();
@@ -113,4 +117,35 @@ fn promotion_flow_resolves_pending_promotion_with_keyboard_choice() {
     assert!(piece_visuals.iter().any(|piece_visual| {
         piece_visual.square == to && piece_visual.piece == Piece::new(Side::White, PieceKind::Queen)
     }));
+}
+
+#[test]
+fn escape_cancels_promotion_overlay_without_leaving_match() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
+    enter_local_match(&mut app);
+
+    let from = Square::from_algebraic("a7").expect("valid square");
+    let to = Square::from_algebraic("a8").expect("valid square");
+    {
+        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
+        match_session.replace_game_state(
+            GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN"),
+        );
+        match_session.selected_square = Some(from);
+        match_session.pending_promotion_move = Some(Move::new(from, to));
+    }
+
+    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
+    app.update();
+    {
+        let mut keyboard_input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
+        keyboard_input.release(KeyCode::Escape);
+        keyboard_input.clear();
+    }
+    app.update();
+
+    assert_eq!(app.world().resource::<MatchSession>().pending_promotion_move, None);
+    assert_eq!(current_state(&app), AppScreenState::InMatch);
 }

```

**Documentation:**

```diff
--- a/crates/game_app/tests/promotion_flow.rs
+++ b/crates/game_app/tests/promotion_flow.rs
@@ -1,3 +1,4 @@
+//! Promotion cancellation stays as a focused shell-flow anchor so the suite proves one real overlay path without replacing logic-level coverage. (ref: DL-003) (ref: DL-004)
 use tempfile::tempdir;
 
 use bevy::prelude::*;

```


**CC-M-003-012** (crates/game_app/tests/app_shell_logic.rs) - implements CI-M-003-012

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/app_shell_logic.rs
@@ -0,0 +1,141 @@
+//! Tests use real game_app types via test_support re-exports so coverage proves the actual
+//! logic module, not a parallel shim universe that can silently drift. (ref: DL-004) (ref: DL-007)
+use chess_core::{
+    AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Move, Side, Square, WinReason,
+};
+use chess_persistence::{
+    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SavedSessionSummary, SaveKind,
+    ShellSettings,
+};
+use game_app::test_support::app_shell_logic;
+use game_app::{
+    AppScreenState, ClaimedDrawReason, ConfirmationKind, MenuContext, RecoveryBannerState,
+    SaveLoadState, ShellMenuState,
+};
+
+#[test]
+fn effective_status_prefers_errors_then_messages_then_recovery_banner() {
+    let save_state = SaveLoadState {
+        last_error: Some(String::from("load failed")),
+        last_message: Some(String::from("saved")),
+        ..Default::default()
+    };
+    assert_eq!(
+        app_shell_logic::effective_shell_status(
+            &ShellMenuState::default(),
+            &save_state,
+            &RecoveryBannerState::default(),
+        )
+        .as_deref(),
+        Some("load failed")
+    );
+
+    let save_state = SaveLoadState {
+        last_message: Some(String::from("saved")),
+        ..Default::default()
+    };
+    let recovery = RecoveryBannerState {
+        available: true,
+        dirty: false,
+        label: Some(String::from("Interrupted Session")),
+    };
+    assert_eq!(
+        app_shell_logic::effective_shell_status(
+            &ShellMenuState::default(),
+            &save_state,
+            &recovery,
+        )
+        .as_deref(),
+        Some("saved")
+    );
+    assert_eq!(
+        app_shell_logic::effective_shell_status(
+            &ShellMenuState::default(),
+            &SaveLoadState::default(),
+            &recovery,
+        )
+        .as_deref(),
+        Some("Interrupted-session recovery is available as Interrupted Session.")
+    );
+}
+
+#[test]
+fn recovery_policy_cycle_and_labels_stay_stable() {
+    assert_eq!(
+        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Resume),
+        RecoveryStartupPolicy::Ask
+    );
+    assert_eq!(
+        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Ask),
+        RecoveryStartupPolicy::Ignore
+    );
+    assert_eq!(
+        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Ignore),
+        RecoveryStartupPolicy::Resume
+    );
+    assert_eq!(
+        app_shell_logic::recovery_policy_label(RecoveryStartupPolicy::Resume),
+        "Resume automatically"
+    );
+    assert_eq!(
+        app_shell_logic::display_mode_label(DisplayMode::Fullscreen),
+        "Fullscreen"
+    );
+    assert_eq!(
+        app_shell_logic::toggle_label("Overwrite Save", true),
+        "Overwrite Save: on"
+    );
+    assert_eq!(
+        app_shell_logic::confirmation_copy(ConfirmationKind::DeleteSave).0,
+        "Delete the selected save?"
+    );
+}
+
+#[test]
+fn result_copy_covers_checkmate_claimed_draw_and_selected_save_lookup() {
+    let white_checkmate = GameStatus::Finished(GameOutcome::Win {
+        winner: Side::White,
+        reason: WinReason::Checkmate,
+    });
+    assert_eq!(
+        app_shell_logic::match_session_result_title(white_checkmate, None),
+        "White Wins"
+    );
+    assert_eq!(
+        app_shell_logic::match_session_result_detail(white_checkmate, None),
+        "Checkmate detected by chess_core."
+    );
+
+    let last_move = Some(Move::new(
+        Square::from_algebraic("e2").unwrap(),
+        Square::from_algebraic("e4").unwrap(),
+    ));
+    assert_eq!(
+        app_shell_logic::derive_save_label(last_move),
+        "Local Match after e2e4"
+    );
+    assert_eq!(
+        app_shell_logic::derive_save_label(None),
+        "Local Match Save"
+    );
+
+    let fivefold = GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
+        AutomaticDrawReason::FivefoldRepetition,
+    )));
+    assert_eq!(
+        app_shell_logic::match_session_result_title(
+            fivefold,
+            Some(ClaimedDrawReason::ThreefoldRepetition),
+        ),
+        "Draw Claimed by Repetition"
+    );
+
+    let save_state = SaveLoadState {
+        manual_saves: vec![SavedSessionSummary {
+            slot_id: String::from("slot-a"),
+            label: String::from("Slot A"),
+            created_at_utc: None,
+            save_kind: SaveKind::Manual,
+        }],
+        settings: ShellSettings {
+            recovery_policy: RecoveryStartupPolicy::Ask,
+            confirm_actions: ConfirmActionSettings::default(),
+            display_mode: DisplayMode::Windowed,
+        },
+        ..Default::default()
+    };
+    let menu_state = ShellMenuState {
+        context: MenuContext::InMatchOverlay,
+        selected_save: Some(String::from("slot-a")),
+        ..Default::default()
+    };
+    assert!(app_shell_logic::return_to_menu_abandons_active_match(
+        AppScreenState::InMatch,
+        &menu_state,
+    ));
+    assert_eq!(
+        app_shell_logic::selected_save_summary(&menu_state, &save_state)
+            .map(|summary| summary.label.as_str()),
+        Some("Slot A")
+    );
+}

```


**CC-M-003-013** (crates/game_app/tests/save_load_logic.rs) - implements CI-M-003-013

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/save_load_logic.rs
@@ -0,0 +1,66 @@
+//! Tests use real game_app types via test_support re-exports so coverage proves the actual
+//! logic module without module-path shims or #[path] includes. (ref: DL-004) (ref: DL-007)
+use chess_persistence::{
+    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SavedSessionSummary, SaveKind,
+    ShellSettings,
+};
+use game_app::test_support::save_load_logic;
+use game_app::{RecoveryBannerState, SaveLoadState};
+
+#[test]
+fn combine_persistence_errors_joins_visible_failures_only() {
+    let combined = save_load_logic::combine_persistence_errors([
+        Some(String::from("save index failed")),
+        None,
+        Some(String::from("settings failed")),
+    ]);
+    assert_eq!(combined.as_deref(), Some("save index failed settings failed"));
+}
+
+#[test]
+fn recovery_visibility_respects_ignore_policy_and_uses_cached_label() {
+    let summary = SavedSessionSummary {
+        slot_id: String::from("recovery"),
+        label: String::from("Interrupted Session"),
+        created_at_utc: None,
+        save_kind: SaveKind::Recovery,
+    };
+    let mut save_state = SaveLoadState {
+        recovery: Some(summary.clone()),
+        settings: ShellSettings {
+            recovery_policy: RecoveryStartupPolicy::Ignore,
+            confirm_actions: ConfirmActionSettings::default(),
+            display_mode: DisplayMode::Windowed,
+        },
+        ..Default::default()
+    };
+    let mut banner = RecoveryBannerState::default();
+
+    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut banner);
+    assert!(!banner.available);
+    assert_eq!(banner.label, None);
+
+    save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
+    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut banner);
+    assert!(banner.available);
+    assert_eq!(banner.label.as_deref(), Some("Interrupted Session"));
+    assert_eq!(
+        save_load_logic::recovery_banner_label(Some(&summary)).as_deref(),
+        Some("Interrupted Session")
+    );
+}
+
+#[test]
+fn save_feedback_messages_and_policy_copy_stay_deterministic() {
+    let summary = SavedSessionSummary {
+        slot_id: String::from("slot-a"),
+        label: String::from("Slot A"),
+        created_at_utc: None,
+        save_kind: SaveKind::Manual,
+    };
+    assert_eq!(save_load_logic::manual_save_message(&summary), "Saved match as Slot A.");
+    assert_eq!(save_load_logic::deleted_save_message("slot-a"), "Deleted save slot-a.");
+    assert!(
+        save_load_logic::recovery_policy_status_copy(RecoveryStartupPolicy::Resume)
+            .contains("MatchLoading")
+    );
+}

```

