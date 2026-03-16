# CLAUDE.md

## Overview

This directory contains the shared CI helper scripts for coverage measurement, packaging, and packaged-boot smoke checks.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `README.md` | Coverage rollout policy, artifact contract, and packaged smoke expectations | Understanding CI helper responsibilities or confirming the honest-coverage workflow |
| `coverage-workspace.sh` | Shared `cargo-llvm-cov` entrypoint, artifact generation, and threshold snapshot export | Running workspace coverage locally or changing CI coverage collection |
| `package-game-app.sh` | Linux staging and tarball assembly for the packaged `game_app` artifact | Changing Linux artifact layout or adjusting which runtime files ship |
| `package-game-app.ps1` | Windows staging and zip assembly for the packaged `game_app` artifact | Changing Windows artifact layout or adjusting which runtime files ship |
| `parse_coverage.py` | Workspace and per-crate coverage summary parsing plus threshold enforcement | Adjusting coverage report parsing or debugging threshold failures |
| `smoke-boot-linux.sh` | Linux packaged-startup smoke script with extraction, `xvfb-run`, timeout, and exit-code checks | Debugging Linux artifact boot failures or changing the packaged smoke contract |
| `smoke-boot-windows.ps1` | Windows packaged-startup smoke script with extraction, process launch, timeout, and cleanup | Debugging Windows artifact boot failures or changing the packaged smoke contract |
