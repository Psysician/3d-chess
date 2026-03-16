# 3D Chess

Rust and Bevy workspace for a desktop 3D chess game.

## Status

M3 completes the local playable game loop and product shell.

- Stack: Rust + Bevy
- Platforms: Windows and Linux
- Scope: standard chess on one 3D board
- Implemented: authoritative standard-chess rules, local playable match flow, manual save/load, interrupted-session recovery, persisted shell settings, and portable CI artifacts with packaged boot smoke
- Deferred beyond M3: Stockfish/UCI-driven matches, installer or signing work, and broader graphics, audio, controls, or accessibility settings

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

## Architecture Boundaries

- `chess_core` stays pure Rust and remains the only gameplay authority for rules, legality, and outcomes.
- `chess_persistence` owns versioned snapshot formats, file-backed repository I/O, platform app-data roots, and the narrow M3 settings contract.
- `game_app` keeps top-level routing coarse and renders menus, pause overlays, promotion UI, save/load flow, and result screens around `MatchSession`.
- `engine_uci` reserves the Stockfish/UCI integration boundary instead of leaking AI concerns into the shipped local shell.

## M3 Invariants

- Save/load restores domain state plus legality-critical shell state; it does not serialize Bevy ECS world state.
- Match launch always flows through an explicit new, load, resume, or rematch intent before entering `InMatch`.
- Manual saves and interrupted-session recovery remain separate user concepts with separate overwrite behavior.
- CI publishes portable Windows and Linux archives and proves packaged boot with smoke scripts; installer and signing work stays outside M3.

## Milestone State

- M0: workspace layout, Bevy shell baseline, CI, toolchain pin, and developer commands.
- M1: `chess_core` legal move generation, move application, check and mate resolution, draw semantics, castling, en passant, promotion, and FEN support.
- M2: `game_app` local match startup, board and piece synchronization, square picking, promotion flow, claim-draw flow, and result transitions.
- M3: `chess_persistence` file-backed saves, recovery state, and shell settings; `game_app` main-menu setup, pause overlays, save/load UX, recovery resume, and result rematch flow; CI packaging and packaged startup smoke on both desktop targets.
