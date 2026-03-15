# CLAUDE.md

## Overview

This directory contains the pure Rust chess domain, including board state, move legality, FEN support, and rules regression tests.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Crate manifest and serde dependency wiring | Changing crate dependencies or package metadata |
| `src/lib.rs` | Module wiring and public re-exports for the domain API | Locating or reshaping the crate surface |
| `src/board.rs` | Board container, piece storage, side filtering, king lookup | Changing board representation or square lookup behavior |
| `src/castling.rs` | Castling-right tracking and FEN castling formatting | Modifying castle legality metadata or castle serialization |
| `src/game.rs` | GameState, FEN parse/serialize, move generation, move application, status evaluation | Implementing or debugging chess rules and state transitions |
| `src/mv.rs` | Public move type and move error surface | Changing move input/output contracts or validation errors |
| `src/pieces.rs` | Side, piece kinds, promotion validity, FEN piece tokens | Modifying piece metadata or piece-level serialization |
| `src/square.rs` | Square coordinates, algebraic conversion, serde string encoding | Debugging square parsing or snapshot serialization |
| `src/status.rs` | Draw availability, automatic draw reasons, game status, outcomes | Changing status reporting or draw semantics |
| `tests/rules.rs` | Scenario coverage for move legality, mate, castling, en passant, promotion, and draw rules | Extending rules coverage or investigating regressions |
