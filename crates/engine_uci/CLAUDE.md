# CLAUDE.md

## Overview

This directory contains the future UCI engine boundary, including request and response types, the controller trait, and deterministic mock-based tests.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and workspace metadata for the UCI seam | Changing package metadata or adding engine-facing dependencies |
| `README.md` | Deterministic testing rationale, invariants, and tradeoffs for the UCI controller seam | Understanding why the crate stays process-free in tests or why the mock is first-class |
| `src/lib.rs` | Module wiring, public re-exports, and the crate-level mock smoke test | Locating the public API or reshaping what downstream crates import |
| `src/controller.rs` | Engine request validation, response shape, error type, and the `EngineController` trait | Changing the engine contract or debugging request and error handling |
| `src/mock.rs` | Scriptable mock controller with health and failure modes for deterministic tests | Extending fake engine behavior or reproducing UCI failures without a subprocess |
| `tests/controller.rs` | Direct controller tests for request validation, success responses, and failure paths | Verifying the engine seam or debugging coverage and regression failures |
