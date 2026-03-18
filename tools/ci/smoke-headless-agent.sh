#!/usr/bin/env bash
set -euo pipefail

smoke_root="$(mktemp -d)"
trap 'rm -rf "${smoke_root}"' EXIT

request='{"command":{"type":"snapshot"}}'
output="$(
    printf '%s\n' "${request}" |
        XDG_DATA_HOME="${smoke_root}" \
        timeout 20s cargo run --quiet -p game_app --features automation-transport --bin game_app_agent
)"

echo "${output}"

printf '%s\n' "${output}" | grep -F '"screen":"main_menu"' >/dev/null
printf '%s\n' "${output}" | grep -F '"error":null' >/dev/null
