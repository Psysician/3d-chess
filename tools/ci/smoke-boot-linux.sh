#!/usr/bin/env bash
set -euo pipefail

archive_path="${1:?archive path required}"
smoke_dir="$(mktemp -d)"
log_path="${smoke_dir}/game_app.log"
trap 'rm -rf "${smoke_dir}"' EXIT

tar -xzf "${archive_path}" -C "${smoke_dir}"
app_dir="$(find "${smoke_dir}" -mindepth 2 -maxdepth 2 -type f -name game_app -printf '%h\n' | head -n 1)"
if [[ -z "${app_dir}" || ! -x "${app_dir}/game_app" ]]; then
    echo "packaged linux archive did not extract to a runnable app directory"
    exit 1
fi

pushd "${app_dir}" >/dev/null
# Force X11 inside xvfb so hosted and local Linux runners do not try a stray Wayland session.
# A timeout-driven pass proves the packaged binary stayed alive long enough to finish startup.
set +e
timeout 15s xvfb-run -a env WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 XDG_SESSION_TYPE=x11 WGPU_BACKEND=gl ./game_app >"${log_path}" 2>&1
status="$?"
set -e

if [[ "${status}" -eq 0 ]]; then
    cat "${log_path}"
    echo "game_app exited before the smoke timeout"
    exit 1
fi

if [[ "${status}" -ne 124 ]]; then
    cat "${log_path}"
    echo "game_app failed during packaged startup smoke"
    exit "${status}"
fi
popd >/dev/null
