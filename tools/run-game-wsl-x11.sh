#!/usr/bin/env bash
set -euo pipefail

# WSL fallback launcher: force winit/Bevy onto X11 by removing WAYLAND_DISPLAY.
# This is useful when a local WSL setup has a flaky Wayland path but a working
# X11/Xwayland bridge.
exec env -u WAYLAND_DISPLAY cargo run -p game_app --bin game_app "$@"
