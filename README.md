# 3D Chess

Repository for a 3D chess game.

## Status

M0 scaffold in progress.

- Stack: Rust + Bevy
- Platforms: Windows and Linux
- Scope: standard chess on one board rendered in 3D
- Modes: local play and Stockfish-backed AI
- Input: mouse and keyboard
- Persistence: save/load
- Quality bar: polished production visuals
- Verification: strong rules coverage and early CI/build checks

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
- `game_app` owns Bevy rendering, UI, input, scene setup, and future orchestration.

## Current M0 Scope

- Scaffold the Rust workspace and crate boundaries.
- Stand up a branded Bevy shell scene with procedural visuals.
- Add Windows/Linux CI for formatting, linting, tests, and release builds.
- Lock the asset directory layout and core developer commands.
