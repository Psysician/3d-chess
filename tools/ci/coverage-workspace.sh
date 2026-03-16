#!/usr/bin/env bash
# One script owns baseline, non-regression, and hard-gate measurement so threshold
# changes never change the instrumentation path or report format. (ref: DL-001) (ref: DL-005) (ref: DL-006)
# Behavior-heavy game_app files stay in scope; only narrow documented exclusions belong here. (ref: DL-002)
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
workspace_root=${1:-$(pwd)}
artifact_dir=${2:-"$workspace_root/target/coverage"}
mode=${COVERAGE_MODE:-baseline}
workspace_threshold=${COVERAGE_WORKSPACE_THRESHOLD:-}
chess_core_threshold=${COVERAGE_CHESS_CORE_THRESHOLD:-}
chess_persistence_threshold=${COVERAGE_CHESS_PERSISTENCE_THRESHOLD:-}
engine_uci_threshold=${COVERAGE_ENGINE_UCI_THRESHOLD:-}
game_app_threshold=${COVERAGE_GAME_APP_THRESHOLD:-}
cargo_llvm_cov_version=${CARGO_LLVM_COV_VERSION:-0.6.16}
ignore_filename_regex=${COVERAGE_IGNORE_FILENAME_REGEX:-"(^|/)crates/game_app/src/main\\.rs$"}

mkdir -p "$artifact_dir"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo install cargo-llvm-cov --locked --version "$cargo_llvm_cov_version"
fi

pushd "$workspace_root" >/dev/null
cargo llvm-cov clean --workspace
cargo llvm-cov test --workspace
cargo llvm-cov report --ignore-filename-regex "$ignore_filename_regex" --json --output-path "$artifact_dir/report.json"
cargo llvm-cov report --ignore-filename-regex "$ignore_filename_regex" --lcov --output-path "$artifact_dir/workspace.lcov"

cat >"$artifact_dir/thresholds.env" <<EOF
COVERAGE_MODE=$mode
COVERAGE_WORKSPACE_THRESHOLD=$workspace_threshold
COVERAGE_CHESS_CORE_THRESHOLD=$chess_core_threshold
COVERAGE_CHESS_PERSISTENCE_THRESHOLD=$chess_persistence_threshold
COVERAGE_ENGINE_UCI_THRESHOLD=$engine_uci_threshold
COVERAGE_GAME_APP_THRESHOLD=$game_app_threshold
COVERAGE_IGNORE_FILENAME_REGEX=$ignore_filename_regex
EOF

python3 "$script_dir/parse_coverage.py" "$artifact_dir/report.json"
popd >/dev/null
