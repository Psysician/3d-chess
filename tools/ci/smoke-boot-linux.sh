#!/usr/bin/env bash
set -euo pipefail

archive_path="${1:?archive path required}"
smoke_dir="$(mktemp -d)"
live_log_path="${smoke_dir}/game_app-live.log"
xvfb_log_path="${smoke_dir}/game_app-xvfb.log"
xvfb_error_path="${smoke_dir}/xvfb.log"
attempt_status=0
live_status=-1
xvfb_status=-1
trap 'rm -rf "${smoke_dir}"' EXIT

tar -xzf "${archive_path}" -C "${smoke_dir}"
app_dir="$(find "${smoke_dir}" -mindepth 2 -maxdepth 2 -type f -name game_app -printf '%h\n' | head -n 1)"
if [[ -z "${app_dir}" || ! -x "${app_dir}/game_app" ]]; then
    echo "packaged linux archive did not extract to a runnable app directory"
    exit 1
fi

run_packaged_app() {
    local log_path="${1:?log path required}"
    shift

    set +e
    timeout 15s "$@" ./game_app >"${log_path}" 2>&1
    attempt_status="$?"
    set -e
}

report_attempt_failure() {
    local label="${1:?label required}"
    local status="${2:?status required}"
    local log_path="${3:?log path required}"

    if [[ -s "${log_path}" ]]; then
        cat "${log_path}"
    fi

    if [[ "${status}" -eq 0 ]]; then
        echo "game_app exited before the smoke timeout (${label})"
    else
        echo "game_app failed during packaged startup smoke (${label})"
    fi
}

pushd "${app_dir}" >/dev/null
# Prefer a usable local X11 display when one already exists, which keeps WSLg and other desktop sessions
# from depending on a second X server under /tmp/.X11-unix. Headless CI runs through xvfb-run instead.
if [[ -n "${DISPLAY:-}" ]]; then
    # Keep the platform-default renderer on a live desktop session so WSLg and similar environments can use
    # whatever backend they actually expose instead of forcing the headless Xvfb OpenGL path.
    run_packaged_app "${live_log_path}" env WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 XDG_SESSION_TYPE=x11
    live_status="${attempt_status}"
    if [[ "${attempt_status}" -eq 124 ]]; then
        popd >/dev/null
        exit 0
    fi
fi

# Force X11 plus the GL renderer inside xvfb so hosted Linux runners do not try a stray Wayland or Vulkan path.
if command -v xvfb-run >/dev/null 2>&1; then
    run_packaged_app "${xvfb_log_path}" xvfb-run -a -e "${xvfb_error_path}" env WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 XDG_SESSION_TYPE=x11 WGPU_BACKEND=gl
    xvfb_status="${attempt_status}"
    if [[ "${attempt_status}" -eq 124 ]]; then
        popd >/dev/null
        exit 0
    fi
fi

if [[ -s "${live_log_path}" ]]; then
    report_attempt_failure "live X11 display" "${live_status}" "${live_log_path}"
fi

if [[ -s "${xvfb_log_path}" ]]; then
    report_attempt_failure "xvfb-run" "${xvfb_status}" "${xvfb_log_path}"
fi

if [[ -s "${xvfb_error_path}" ]]; then
    cat "${xvfb_error_path}"
fi

if [[ -s "${xvfb_error_path}" ]] && grep -Fq 'Mode of /tmp/.X11-unix should be set to 1777' "${xvfb_error_path}"; then
    echo "xvfb-run could not start because /tmp/.X11-unix is not sticky; a live X11 display or corrected directory mode is required"
fi

popd >/dev/null
if [[ "${xvfb_status}" -ge 0 ]]; then
    attempt_status="${xvfb_status}"
elif [[ "${live_status}" -ge 0 ]]; then
    attempt_status="${live_status}"
else
    echo "packaged linux smoke could not find a live X11 display or xvfb-run"
    exit 1
fi

if [[ "${attempt_status}" -eq 0 ]]; then
    exit 1
fi

exit "${attempt_status}"
