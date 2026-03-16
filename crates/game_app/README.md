# game_app

`game_app` is the Bevy-facing orchestration layer for the project. M3 turns the playable M2 match loop into a complete local product shell without moving chess authority out of `chess_core`.

## M3 Shell Architecture

- `MatchSession` is the only resource allowed to bridge Bevy and `chess_core`.
- `AppScreenState` stays intentionally coarse: `Boot -> MainMenu -> MatchLoading -> InMatch -> MatchResult`.
- `MatchLaunchIntent` and `PendingLoadedSnapshot` carry new, load, resume, and rematch requests through `MatchLoading` so the shell never guesses which path the player intended.
- `MenuPlugin` owns main-menu setup, in-match pause overlays, load-list state, settings state, and destructive confirmations as modal resources instead of top-level routes.
- `SaveLoadPlugin` owns `SessionStore` access, startup recovery policy, manual save and load requests, autosave-backed interrupted-session recovery, and the shipped settings trio.
- `AppShellPlugin` renders the main menu, pause surfaces, promotion overlay, and result flow from modal shell resources while board and piece rendering stay in dedicated scene plugins.
- Board rendering, piece placement, and cursor picking all share the same coordinate helpers in `board_coords.rs`.

## Invariants

- `chess_core::GameState` remains the source of truth for legal moves, side to move, checks, and terminal outcomes.
- Bevy entities mirror domain state; they do not own chess legality.
- Match launch always enters `InMatch` through an explicit launch intent consumed by `MatchLoading`.
- Save/load restores only domain state plus legality-critical shell interaction state such as pending promotion, selected square, claimed draw context, and recovery dirtiness.
- The setup and pause surfaces stay orthogonal to `AppScreenState`; opening them does not leave `InMatch`.
- Manual saves and interrupted-session recovery remain separate user concepts with separate overwrite and clearing behavior.
- Board square identity must remain stable across rendering, picking, highlighting, and tests.
- AI match flow, richer settings, and release engineering beyond portable archives stay outside the shipped M3 shell.

## Tradeoffs

- Coarse screen states plus modal resources keep shell growth manageable without creating one app state per popup.
- Repository I/O and recovery policy stay in dedicated plugins instead of leaking into `AppShellPlugin` or `MatchSession`.
- Only three settings persist in M3: startup recovery behavior, destructive-action confirmations, and display mode.
- The board and pieces stay procedural so effort goes into interaction correctness, persistence flow, and test coverage instead of asset pipelines.
- Picking uses internal camera-ray math because the board plane is fixed and deterministic; a generic picking dependency would add surface area without helping the current use case.
- Claimable draws are resolved in `MatchSession` so the shell can end claimable games without widening `chess_core` beyond the persisted session contract it already owns.
