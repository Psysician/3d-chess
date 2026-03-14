# 3D Chess Milestones

Status: Locked
State: Planning baseline approved

## Confirmed Constraints

- The project is a 3D chess game built from scratch in this repository.
- The runtime and delivery stack must not be JavaScript, TypeScript, or web-based.
- Planning should happen before implementation.
- The repository is currently greenfield aside from a minimal README.

## Locked Product and Stack Decisions

- Game definition: standard chess on a single board rendered in 3D
- Platforms: Windows and Linux
- Engine/runtime: Rust with Bevy
- Primary language: Rust
- First playable release: local play plus Stockfish-backed AI
- AI strategy: integrate a Stockfish-level engine rather than building a custom chess engine
- Presentation target: polished production visuals from the start
- Testing posture: strong chess-rules test coverage from day one plus CI/build verification early
- Input scope: mouse and keyboard only
- Persistence scope: save/load only

## Architecture Baseline

- Use a Cargo workspace instead of a single crate.
- Keep the chess rules engine pure Rust and independent from Bevy.
- Keep rendering, input, animation, and audio inside the Bevy app layer.
- Integrate Stockfish through a dedicated adapter boundary rather than mixing UCI process concerns into gameplay code.
- Save domain snapshots, not raw ECS world state.
- Treat visual quality as a first-class milestone input rather than a final pass only.

## Provisional Milestones

### M0. Product and Stack Lock

- Lock the Cargo workspace structure and project conventions.
- Pin the initial Rust toolchain and Bevy release line for project bootstrap.
- Bootstrap formatting, linting, tests, CI, asset layout, and build commands.
- Produce an art-direction baseline covering board, materials, lighting, camera, and UI tone.

Acceptance gate:

- `cargo fmt`, `cargo clippy`, `cargo test`, and native release builds run in CI on Windows and Linux.
- The Bevy application opens into a branded shell scene with production-intent visual direction.

### M1. Chess Core

- Implement board state, turns, legal move generation, captures, check, checkmate, stalemate, castling, en passant, and promotion.
- Add internal serialization support required for save/load and engine communication.
- Build exhaustive automated tests for rules correctness before UI behavior depends on the core.

Acceptance gate:

- The domain crate passes strong unit and scenario coverage for legal move generation and end-state detection.
- A saved match snapshot restores the full domain state without Bevy-specific state leakage.

### M2. 3D Board Interaction

- Build the 3D board scene, piece representations, camera rig, selection, move preview, and move execution flow.
- Add highlighting, animation systems, input handling, and polished baseline materials/lighting.
- Reach a complete local human-vs-human playable match inside the production app shell.

Acceptance gate:

- A user can complete a full local match with mouse/keyboard only.
- The board, pieces, UI overlays, and transitions meet the polished visual target rather than placeholder quality.

### M3. Playable Game Loop

- Add menu flow, game state transitions, result handling, restart flow, settings, and save/load UX.
- Harden promotion flow, check state feedback, invalid move behavior, and recovery from interrupted sessions.
- Package a complete local playable build for Windows and Linux.

Acceptance gate:

- Save/load works from the shipped UI and preserves an active match correctly.
- Windows and Linux artifacts boot successfully from CI-produced builds.

### M4. Opponent Layer

- Add the Stockfish integration boundary, engine lifecycle management, and AI game flow.
- Tune difficulty controls, move-time behavior, and failure handling when the engine is missing or unhealthy.
- Validate the engine adapter against the rules layer and game shell.

Acceptance gate:

- A user can start and finish a local match against Stockfish-backed AI.
- Engine integration survives process restarts and reports actionable errors in the UI.

### M5. Presentation and Content Polish

- Refine models, materials, lighting, audio, UI, piece motion, camera moves, and feedback effects.
- Add onboarding, accessibility-minded presentation options, and quality-of-life details.
- Close gaps between functional scenes and release-candidate presentation quality.

Acceptance gate:

- The main game loop, menus, and AI match flow share a coherent production visual language.
- Major usability gaps are closed without expanding scope into controller support or online play.

### M6. Release Engineering

- Finalize export targets, packaging, versioning, QA checklist, and project documentation.
- Harden CI, release automation, and distribution requirements for the selected Stockfish packaging strategy.
- Prepare distributable builds for Windows and Linux.

Acceptance gate:

- Release documentation covers app packaging, save compatibility expectations, and Stockfish distribution requirements.
- The repository can produce reproducible release candidates for both target platforms.

## Dependency Order

1. M0 before all other milestones
2. M1 before M2, M3, and M4
3. M2 before M3 and M5
4. M3 and M4 before M5
5. M5 before M6

## Planning References

- Detailed implementation plan: [plans/implementation-plan.md](/home/franky/repos/3d-chess/plans/implementation-plan.md)
