# CLAUDE.md

## Overview

This directory contains the packaging and packaged-boot smoke scripts for desktop CI artifacts.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `package-game-app.sh` | Linux staging and tarball assembly for the packaged `game_app` artifact | Changing Linux artifact layout or adjusting which runtime files ship |
| `package-game-app.ps1` | Windows staging and zip assembly for the packaged `game_app` artifact | Changing Windows artifact layout or adjusting which runtime files ship |
| `smoke-boot-linux.sh` | Linux packaged-startup smoke script with extraction, `xvfb-run`, timeout, and exit-code checks | Debugging Linux artifact boot failures or changing the packaged smoke contract |
| `smoke-boot-windows.ps1` | Windows packaged-startup smoke script with extraction, process launch, timeout, and cleanup | Debugging Windows artifact boot failures or changing the packaged smoke contract |
