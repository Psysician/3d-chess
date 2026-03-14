# Rust + Bevy 3D Chess Implementation Plan

Status: Locked
Primary targets: Windows, Linux
Stack: Rust, Bevy, Stockfish via adapter boundary

## Product Goal

Build a desktop-native 3D chess game that renders standard chess on a single 3D board, supports local play and Stockfish-backed AI, ships on Windows and Linux, and reaches a polished presentation standard without relying on JavaScript, TypeScript, or web delivery.

## Scope

In scope:

- Standard chess rules
- 3D board and piece presentation
- Mouse and keyboard input
- Local human-vs-human play
- Local human-vs-AI play through Stockfish integration
- Save/load for in-progress matches
- Strong rules-engine tests
- Early CI and cross-platform build verification

Out of scope for the first plan:

- Online multiplayer
- Controller support
- User-facing PGN/FEN import/export
- Custom chess variants
- Building a bespoke chess AI

## Architecture Decisions

### 1. Cargo Workspace, Not a Monolith

Use a Cargo workspace so the rules domain, Bevy app shell, persistence layer, and engine adapter remain isolated.

Recommended structure:

```text
3d-chess/
  Cargo.toml
  rust-toolchain.toml
  assets/
    audio/
    fonts/
    materials/
    models/
    scenes/
    textures/
  crates/
    chess_core/
    chess_persistence/
    engine_uci/
    game_app/
  tests/
    integration/
  tools/
    ci/
    asset_pipeline/
  .github/
    workflows/
```

### 2. Pure Domain Core

`chess_core` owns the authoritative rules model:

- board state
- piece state
- turn and move application
- legal move generation
- check, checkmate, stalemate, repetition, fifty-move tracking if included
- castling, en passant, promotion
- internal FEN-like serialization used for engine communication and save snapshots

`chess_core` must not depend on Bevy. This keeps testing cheap, deterministic, and fast.

### 3. Bevy Owns Presentation and App Flow

`game_app` owns:

- scene setup
- camera rig
- board and piece spawning
- materials and lighting
- animation and VFX hooks
- menus and overlays
- selection and input interpretation
- app state transitions
- audio playback

The Bevy world mirrors domain state but does not become the source of truth for chess rules.

### 4. Stockfish Through a UCI Adapter Boundary

`engine_uci` manages a Stockfish subprocess through the UCI protocol rather than linking gameplay directly to engine internals.

Why this is the right default:

- It matches Stockfish's canonical integration surface.
- It keeps engine lifecycle, time controls, and process recovery isolated.
- It reduces coupling between gameplay and engine packaging.
- It keeps licensing and distribution choices cleaner than directly embedding engine code.

Design shape:

- `EngineController` trait
- `StockfishProcessController` implementation
- request/response message model for position, search, stop, and best move
- health checks, startup handshake, restart policy, timeout policy

### 5. Save Domain Snapshots, Not ECS State

`chess_persistence` stores versioned match snapshots derived from domain state plus minimal UX settings.

Save payload should include:

- board position
- side to move
- move history needed for legal-state reconstruction
- promotion state if pending
- AI match configuration when relevant
- save format version

Save payload should not include:

- raw entity IDs
- transient animation state
- renderer-specific handles

### 6. Production Visuals Start Early

The project should not wait until the end for visual quality. M0 and M2 should establish:

- board scale and silhouette
- camera framing
- material language
- lighting direction
- UI styling
- piece readability against the board
- animation tone

This prevents a strong rules prototype from turning into a visual rewrite later.

## Recommended Crate Responsibilities

### `crates/chess_core`

- Pure data structures and rules
- Move generation
- Match lifecycle
- Domain serialization helpers
- Unit tests and scenario tests

### `crates/chess_persistence`

- Versioned save schema
- Snapshot encode/decode
- Backward-compatibility handling for future save versions

### `crates/engine_uci`

- UCI protocol process management
- Stockfish discovery and configuration
- Engine request queueing
- Failure handling and restart logic
- Mock engine implementation for tests

### `crates/game_app`

- Bevy app entrypoint
- plugins for board scene, UI, input, animation, audio, save/load, and AI orchestration
- asset loading and runtime configuration

## Bevy App Structure

Recommended plugin split:

- `app_shell_plugin`
- `board_scene_plugin`
- `piece_view_plugin`
- `input_plugin`
- `move_feedback_plugin`
- `menu_plugin`
- `save_load_plugin`
- `ai_match_plugin`
- `audio_plugin`

Recommended Bevy states:

- `Boot`
- `MainMenu`
- `LocalSetup`
- `MatchLoading`
- `InMatch`
- `Paused`
- `MatchResult`

## Testing Strategy

### Core Rules

Make `chess_core` the most heavily tested part of the codebase.

Minimum coverage focus:

- per-piece legal moves
- capture rules
- check detection
- pinned piece behavior
- castling legality
- en passant legality
- promotion resolution
- checkmate and stalemate detection
- save/load round-trip correctness

Recommended test mix:

- unit tests for move generation helpers
- scenario tests for whole-board positions
- regression fixtures for historical bugs
- property-style tests where useful for invariants such as move application and reversal safety

### Engine Integration

Use two layers:

- deterministic tests against a fake UCI engine process
- opt-in smoke tests against a real Stockfish binary in CI

This avoids making the whole test suite dependent on external engine timing.

### App and Build Verification

CI should validate:

- formatting
- linting
- workspace tests
- release builds for Windows and Linux
- asset-path and save-schema sanity checks

## CI Plan

Set up GitHub Actions early with a Windows and Linux matrix.

Initial CI lanes:

1. `fmt`
   - `cargo fmt --check`
2. `lint`
   - `cargo clippy --workspace --all-targets -- -D warnings`
3. `test`
   - `cargo test --workspace`
4. `build`
   - release build for `game_app` on Windows and Linux
5. `engine-smoke`
   - download or provision Stockfish
   - verify UCI handshake and one move request

## Stockfish Packaging Strategy

Default plan:

- integrate via external Stockfish executable
- resolve the engine path through configuration and platform-aware defaults
- allow bundling in release packages once licensing and distribution docs are in place

Operational requirements:

- clear engine-not-found error handling
- startup timeout and retry policy
- UI messaging when the engine is unavailable

Licensing note:

- Treat Stockfish packaging as a release-engineering concern from the start, not a last-minute add-on.
- Keep the adapter boundary clean so packaging strategy can evolve without rewriting gameplay systems.

## Milestone Breakdown

### M0. Workspace and Visual Baseline

Deliver:

- Cargo workspace and crate skeleton
- Rust toolchain pin
- Bevy bootstrap app
- CI for Windows and Linux
- asset directory layout
- visual direction brief

Definition of done:

- empty shell app runs locally
- CI passes on both target OSes
- project conventions are written down

### M1. Chess Domain and Persistence Foundation

Deliver:

- complete rules domain in `chess_core`
- internal position serialization for saves and engine communication
- versioned save schema in `chess_persistence`
- heavy automated test coverage

Definition of done:

- legal chess rules are enforced in tests
- save/load domain round-trips are stable

### M2. 3D Playable Local Match

Deliver:

- polished board scene
- piece assets or placeholders at production-intent quality
- move selection and execution flow
- local human-vs-human gameplay
- match HUD and feedback

Definition of done:

- a full local match is playable end-to-end
- scene quality is aligned with the intended ship direction

### M3. App Shell and Save/Load UX

Deliver:

- menus
- match setup flow
- save/load UI
- pause/restart/result screens
- recovery from interrupted sessions

Definition of done:

- a saved game can be resumed through shipped UI flows
- the app behaves like a complete product shell rather than a single scene

### M4. Stockfish AI Integration

Deliver:

- UCI adapter
- Stockfish process lifecycle management
- AI match mode
- difficulty and think-time controls
- engine fault handling

Definition of done:

- a user can play a full match against AI
- engine failures degrade gracefully

### M5. Release-Candidate Polish

Deliver:

- final presentation pass
- audio polish
- camera and animation tuning
- usability cleanup
- accessibility-minded options appropriate to scope

Definition of done:

- the full app feels coherent and shippable
- no major mismatch remains between visual direction and implemented scenes

### M6. Release Engineering

Deliver:

- packaging workflow
- release notes template
- QA checklist
- Stockfish distribution documentation
- reproducible release process

Definition of done:

- Windows and Linux release candidates can be built consistently
- packaging docs cover save expectations and Stockfish requirements

## Key Risks and Mitigations

### Risk: Bevy upgrade churn

Mitigation:

- pin the Bevy release line in M0
- avoid upgrading Bevy mid-milestone
- isolate Bevy-specific code inside `game_app`

### Risk: Stockfish distribution complexity

Mitigation:

- integrate through a process boundary first
- document engine packaging requirements in M6
- keep engine discovery configurable per platform

### Risk: Production visual target slows systems work

Mitigation:

- lock an art direction baseline in M0
- build a polished vertical slice by M2
- avoid placeholder-heavy scene architecture that will be discarded later

### Risk: Cross-platform drift

Mitigation:

- validate Windows and Linux in CI from the first bootstrap milestone
- avoid filesystem and process-launch assumptions that differ by OS

## Immediate Execution Order

1. Create the Cargo workspace and crate boundaries.
2. Add CI, formatting, linting, and test scaffolding.
3. Build `chess_core` before any serious Bevy gameplay logic.
4. Build the polished board interaction vertical slice on top of the tested domain.
5. Add save/load UX and Stockfish integration after the local game loop is stable.

## Notes for Implementation Start

- Internal FEN support is worth implementing even without user-facing FEN import/export because UCI engine integration benefits from it immediately.
- The app should treat AI turns as asynchronous work so the render loop remains responsive.
- Engine communication logs should be easy to enable in debug builds and quiet in release builds.
