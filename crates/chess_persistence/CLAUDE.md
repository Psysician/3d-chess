# CLAUDE.md

## Overview

This directory contains the persistence crate that stores saved sessions, recovery state, and shell settings for 3D Chess.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and dependencies for snapshots, app-data roots, timestamps, and atomic file writes | Changing persistence dependencies or package metadata |
| `README.md` | Persistence rationale, invariants, and tradeoffs for saves, recovery, and settings | Understanding why the crate is domain-first and why recovery stays separate from manual saves |
| `src/lib.rs` | Public snapshot and store exports plus legality-preserving snapshot tests | Understanding the crate surface or extending persistence verification |
| `src/snapshot.rs` | Versioned snapshot schema, shell-state payloads, and restore helpers | Modifying the persisted session contract or adding legality-critical shell metadata |
| `src/store.rs` | `SessionStore`, app-data path policy, atomic JSON I/O, save summaries, and settings types | Changing repository behavior, slot handling, or runtime storage layout |
| `tests/session_store.rs` | Real-filesystem coverage for manual saves, recovery records, settings, and slot-id hardening | Verifying repository behavior or debugging on-disk persistence regressions |
