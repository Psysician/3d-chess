# game_app

`game_app` is the Bevy-facing orchestration layer for the project. M2 turns it from a static shell into a complete local human-vs-human match loop while still keeping chess rules inside `chess_core`.

## M2 Architecture

- `MatchSession` is the only resource allowed to bridge Bevy and `chess_core`.
- Board rendering, piece placement, and cursor picking all share the same coordinate helpers in `board_coords.rs`.
- The shell flow is intentionally narrow: `MainMenu -> MatchLoading -> InMatch -> MatchResult`.
- Promotion and draw claims are treated as interaction state owned by `game_app`, not as ad hoc ECS rules.

## Invariants

- `chess_core::GameState` remains the source of truth for legal moves, side to move, checks, and terminal outcomes.
- Bevy entities mirror domain state; they do not own chess legality.
- Board square identity must remain stable across rendering, picking, highlighting, and tests.
- M2 stops at local play. Save/load shell UX, broader menu expansion, and AI integration remain later milestones.

## Tradeoffs

- The board and pieces stay procedural in M2 so effort goes into interaction correctness, feedback, and test coverage instead of asset pipelines.
- Picking uses internal camera-ray math because the board plane is fixed and deterministic; a generic picking dependency would add surface area without helping the current use case.
- Claimable draws are resolved in `MatchSession` so the shell can end claimable games now without widening `chess_core`'s persisted game-state model during M2.
