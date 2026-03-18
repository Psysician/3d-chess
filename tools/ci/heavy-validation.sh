#!/usr/bin/env bash
set -euo pipefail

cargo test -p game_app --features automation-transport --test binary_target --test automation_transport --test real_world_rounds
bash tools/ci/smoke-headless-agent.sh
