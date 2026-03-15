# CI Packaging Notes

Portable artifact packaging and smoke-start verification for `game_app`.

## Architecture

- Packaging scripts stage the release binary with the runtime `assets/` tree into a single top-level app directory.
- Smoke scripts extract that directory and treat surviving a bounded startup window as proof of a bootable artifact. (ref: DL-006)

## Invariants

- CI proves packaged runtime boot, not just successful compilation. (ref: DL-006)
- Windows and Linux archives stay self-contained with one top-level app directory so the workflow can upload runnable artifacts. (ref: DL-006)

## Runner Expectations

- Linux smoke uses `xvfb-run` to provide a stable desktop surface on hosted runners.
- Windows smoke starts the extracted executable directly from the staged app directory.
