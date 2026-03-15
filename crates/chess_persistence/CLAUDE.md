# CLAUDE.md

## Overview

This directory contains the versioned save boundary that persists and restores `chess_core::GameState`.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and dependencies on `chess_core` and serde JSON | Changing persistence dependencies or package metadata |
| `src/lib.rs` | Public snapshot exports and legality-preserving round-trip test | Understanding the crate surface or extending persistence verification |
| `src/snapshot.rs` | Save format version, snapshot metadata, and game-state restore helpers | Modifying save schema or restoring richer match state |
