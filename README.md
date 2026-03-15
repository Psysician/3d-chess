# 3D Chess

Repository for a 3D chess game.

## Status

M2 local board interaction and playable match flow complete.

- Stack: Rust + Bevy
- Platforms: Windows and Linux
- Scope: standard chess on one board rendered in 3D
- Modes: local play and Stockfish-backed AI
- Input: mouse and keyboard
- Persistence: save/load
- Quality bar: polished production visuals
- Verification: strong rules coverage and early CI/build checks
- Implemented: full standard-chess domain rules, versioned match snapshots, and a local playable Bevy match loop with promotion, claim-draw flow, and result transitions
- Current app shell: procedurally rendered 3D board driven by `chess_core` through `MatchSession`

## Planning Docs

- [Milestones](/home/franky/repos/3d-chess/plans/milestones.md)
- [Implementation Plan](/home/franky/repos/3d-chess/plans/implementation-plan.md)

## Workspace Layout

```text
3d-chess/
  Cargo.toml
  rust-toolchain.toml
  assets/
  crates/
    chess_core/
    chess_persistence/
    engine_uci/
    game_app/
  plans/
  tools/
```

## Toolchain

- Rust `1.93.0`
- Bevy `0.17.2`

## Commands

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p game_app`
- `cargo build --workspace --release`

## Milestone Boundaries

- `chess_core` stays pure Rust and owns the authoritative chess domain.
- `chess_persistence` owns versioned save boundaries and snapshot formats.
- `engine_uci` reserves the Stockfish/UCI integration boundary.
- `game_app` owns Bevy rendering, UI, input, scene setup, and match orchestration.

## Current State

- M0 is complete: workspace layout, Bevy shell baseline, CI, toolchain pin, and developer commands are in place.
- M1 is complete: `chess_core` now owns legal move generation, move application, check/checkmate/stalemate, castling, en passant, promotion, draw semantics, and exact FEN support.
- M2 is complete: `game_app` now starts local matches, syncs board/piece presentation from `GameState`, supports mouse square selection plus keyboard promotion shortcuts, exposes claim-draw flow, and routes into result/rematch states.
- `chess_persistence` still snapshots full `GameState` so save/load can preserve legality-critical state once later shell work lands.
