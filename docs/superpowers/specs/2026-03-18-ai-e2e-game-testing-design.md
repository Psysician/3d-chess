# AI-Driven E2E Game Testing

## Goal

Build a two-layer testing harness that plays complete chess games to find rule bugs, verify game flow integrity, and stress-test the engine. Layer 1 runs thousands of fast games at the `chess_core` domain level. Layer 2 plays a handful of complete games through the `game_app` automation harness to test the full Bevy stack.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Move selection | Scripted sequences + random fill | Scripted games cover known edge cases; random games explore states humans rarely reach |
| Reporting | Panic for `cargo test` + JSON for CI | Fast feedback in development, structured artifacts for batch analysis |
| Scale | Configurable via env var | Small default (20 games) for `cargo test`, scales to thousands for nightly/stress runs |
| Verification scope | Checkmate, stalemate, draw rules, move legality invariants | Coverage of all implemented rules after every move. Note: insufficient material draws (K vs K, K+B vs K, etc.) are not yet implemented in the engine and therefore not tested. |

## Architecture

Two layers with shared infrastructure:

```
chess_core (Layer 1 - Rules Oracle)
  ├── src/testing.rs          # MoveStrategy trait, strategies, GameOracle, InvariantChecker, reporting
  └── tests/game_oracle.rs    # Scripted scenarios + random game batches

game_app (Layer 2 - Full-Stack Playthrough)
  ├── src/automation.rs        # Add ClaimDraw variant to AutomationMatchAction (prerequisite)
  └── tests/automation_game_playthrough.rs   # Complete games through AutomationHarness
```

Layer 1 depends only on `chess_core`. Layer 2 depends on `game_app` (which transitively includes `chess_core`). The `MoveStrategy` trait and `InvariantChecker` live in `chess_core` so both layers share them.

## Section 1: Shared Infrastructure

### Location

`chess_core/src/testing.rs` — public module gated behind `#[cfg(feature = "test-support")]`.

Why a feature flag and not `#[cfg(test)]`: `cfg(test)` only applies when compiling the crate itself for testing, not when it is a dependency of another crate under test. Since `game_app` tests need to use these types, a feature flag is required.

### MoveStrategy Trait

```rust
pub trait MoveStrategy {
    fn select_move(&mut self, game: &GameState, legal_moves: &[Move]) -> Move;
}
```

### Built-in Strategies

**`RandomStrategy`** — Uniform random selection from legal moves. Accepts a seed for reproducibility.

**`WeightedStrategy`** — Biases toward captures, promotions, en passant, and castling using static weights. Weights are implementation-defined and documented in the code. Does not detect checks (which would require applying each candidate move + `is_in_check`, doubling per-move cost at scale). Concrete baseline weights:

- Capture: 3x base weight
- Promotion: 5x base weight
- En passant: 4x base weight
- Castling: 2x base weight
- Quiet move: 1x base weight

**`ScriptedStrategy`** — Takes a `Vec<Move>`, plays them in order. Falls back to `RandomStrategy` when the script is exhausted. Supports patterns like "play Scholar's Mate opening, then random continuation."

### GameOracle

Plays a complete game using one strategy per side:

```rust
pub struct GameOracle {
    white: Box<dyn MoveStrategy>,
    black: Box<dyn MoveStrategy>,
    max_moves: u16,  // default 500
}

impl GameOracle {
    pub fn play_game(&mut self, initial: GameState) -> GameRecord { ... }
}
```

#### Game Loop

The `play_game` method follows this loop:

```
1. Check game.status() — if Finished, record outcome and return.
2. Call game.legal_moves() — if empty, the status check above should have caught it (this is itself a violation if it happens with Ongoing status).
3. Call strategy.select_move(game, &legal_moves) to pick a move.
4. Call game.apply_move(chosen_move):
   - On Ok(next_state): run InvariantChecker on (game, chosen_move, next_state), record the move, advance.
   - On Err(e): record a violation — a move from legal_moves() was rejected by apply_move(), which indicates an engine bug. Continue with a different legal move if possible.
5. If move_count >= max_moves, stop and record outcome as MoveLimitReached.
```

### GameRecord

```rust
pub enum GameTermination {
    Completed(GameStatus),    // Normal game end (checkmate, stalemate, draw)
    MoveLimitReached(u16),    // Game capped at max_moves without terminal position
}

pub struct GameRecord {
    pub initial_fen: String,
    pub moves: Vec<Move>,
    pub final_fen: String,
    pub termination: GameTermination,
    pub violations: Vec<Violation>,
    pub move_count: u16,
}
```

A `MoveLimitReached` termination is not a violation — it is an expected outcome for random games that shuffle pieces without progress. It is excluded from violation analysis. Only games that terminate via `Completed` have their final status cross-checked.

### InvariantChecker

Runs after every `apply_move` call. Receives the state before the move, the move itself, and the state after the move. Checks:

1. **King safety** — The side that just moved does not have its king in check (the engine should prevent this, but we verify).
2. **King count** — Exactly one king per side at all times.
3. **Piece count consistency** — Per-side, per-type accounting:
   - On a non-promotion, non-capture move: piece counts unchanged.
   - On a capture: the captured side loses exactly one piece of the captured type.
   - On a promotion: the promoting side loses one pawn and gains one piece of the promotion type. If the promotion is also a capture, the captured side loses one piece.
   - Total piece count never increases.
4. **Castling rights monotonicity** — Castling rights only decrease, never increase.
5. **En passant validity** — If en passant target is set, it is on the correct rank (0-indexed rank 2 for Black's target, 0-indexed rank 5 for White's target, matching the `Square::rank()` convention in `game.rs`), and a pawn of the correct side is on the adjacent rank.
6. **Halfmove clock** — Resets to 0 on pawn moves and captures, increments by 1 otherwise.
7. **Fullmove number** — Increments by 1 after Black's move, stays the same after White's move.
8. **Position history** — Tracks correctly for repetition detection.

### Lint Compatibility

The workspace sets `unwrap_used = "deny"`. All code in the testing module uses `expect("descriptive message")` instead of `unwrap()`. For `rand` APIs that return `Option` (e.g., `choose()`), use `expect("legal_moves is non-empty")`.

### New Dependencies

The `test-support` feature in `chess_core/Cargo.toml` enables the `testing` module and adds `rand` + `serde`/`serde_json` as dependencies (not dev-dependencies, since downstream crates like `game_app` need to use the module in their tests too).

- `rand` — new dependency behind `test-support` feature
- `serde` and `serde_json` — already available in the workspace

## Section 2: Layer 1 — Rules Oracle

### Location

`chess_core/tests/game_oracle.rs`

### Scripted Scenarios

These always run regardless of configuration:

| Scenario | Moves | Purpose |
|----------|-------|---------|
| Scholar's Mate | 4-move checkmate | Checkmate detection, game termination |
| Fool's Mate | 2-move checkmate | Fastest possible checkmate |
| En passant capture | Scripted to reach en passant position | En passant mechanics end-to-end |
| Kingside + queenside castling | Both sides castle | Castling rights, rook repositioning |
| Pawn promotion (all 4 types) | Scripted to reach 8th rank | Promotion to Q/R/B/N |
| Stalemate via scripted play | Known stalemate sequence | Stalemate vs checkmate distinction |
| 3-fold repetition draw claim | Repeated knight shuttling | Draw availability detection |
| 50-move rule | FEN with halfmove clock at 100 | Halfmove clock tracking |
| En passant exposing king (illegal) | Pinned pawn en passant | Pin detection through en passant |
| Double check king-only moves | FEN with double check | Double check constraint enforcement |

### Random Game Batches

Configurable via environment variable:

- **Default:** `CHESS_ORACLE_GAMES=20` (~1 second)
- **CI:** `CHESS_ORACLE_GAMES=500`
- **Stress:** `CHESS_ORACLE_GAMES=5000`

Each game uses a seeded RNG derived from `base_seed + game_index`. The base seed defaults to a fixed value for deterministic CI, overridable via `CHESS_ORACLE_SEED=<u64>`.

Games alternate between `RandomStrategy` and `WeightedStrategy` across the batch (even-indexed games use random, odd-indexed use weighted).

Invariant checking runs after every single move in every game.

## Section 3: Layer 2 — Full-Stack Playthrough

### Location

`game_app/tests/automation_game_playthrough.rs`

### Prerequisite: Add `AutomationMatchAction::ClaimDraw`

The current `AutomationMatchAction` enum lacks a `ClaimDraw` variant. Draw claims currently work only through a Bevy UI button press in `move_feedback.rs` calling `match_session.claim_draw()` directly. To test draw claims through automation, add:

```rust
// In AutomationMatchAction:
ClaimDraw,
```

The dispatch arm in `apply_match_action` calls `match_session.claim_draw()`, matching the existing UI button behavior. Files modified:

- `game_app/src/automation.rs` — add variant to `AutomationMatchAction`
- `game_app/src/plugins/automation.rs` — add dispatch arm in `apply_match_action`
- `game_app/src/automation_transport.rs` — add serde support (if `automation-transport` feature)

### Scenarios

| Scenario | What it tests |
|----------|---------------|
| Complete game to checkmate | `StartNewMatch` -> scripted Scholar's Mate via `SubmitMove` -> verify screen transitions to `MatchResult` -> verify snapshot reports correct winner |
| Mid-game save/load | Play 5 moves -> `SaveManual` -> `ReturnToMenu` -> load the save -> verify FEN matches -> continue to completion |
| Recovery resume | Pre-populate recovery snapshot (matching existing test pattern in `automation_semantic_flow.rs`) with a FEN reflecting 3 moves played -> new harness boots -> verify recovery banner -> resume -> verify FEN matches -> play to completion |
| Promotion through automation | Scripted promotion position -> `SubmitMove` with promotion piece -> verify piece type in snapshot |
| Draw claim flow | Play to claimable draw position -> verify `draw_available` in snapshot -> `ClaimDraw` via automation -> verify `MatchResult` with `GameOutcome::Draw(DrawReason::...)` outcome |
| Rematch after completion | Play to checkmate -> `Rematch` -> verify fresh starting position -> play a few moves |
| Stalemate through full stack | Scripted stalemate -> verify `GameOutcome::Draw(DrawReason::Stalemate)` in snapshot |

### Scale

Fixed at 7 scenarios. These are deterministic integration tests, not fuzzing. They exercise the automation command pipeline, snapshot accuracy, persistence round-trips, and screen state transitions.

## Section 4: JSON Reporting

### Report Structure

`GameReport` is the serialized form of `GameRecord`, adding metadata for batch analysis:

```rust
pub struct GameReport {
    pub seed: u64,
    pub initial_fen: String,
    pub moves: Vec<MoveRecord>,
    pub outcome: String,
    pub violations: Vec<ViolationRecord>,
    pub total_moves: u16,
    pub strategy: String,
    pub timestamp_utc: String,
}

pub struct MoveRecord {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
    pub fen_after: String,
}

pub struct ViolationRecord {
    pub move_number: u16,
    pub fen_before: String,
    pub attempted_move: String,
    pub violation: String,
}
```

Conversion: `GameReport::from_record(record: &GameRecord, seed: u64, strategy: &str) -> Self` adds the metadata fields.

### Output Directories

- Layer 1: `target/test-reports/oracle/`
- Layer 2: `target/test-reports/playthrough/`

### Modes

**Default (`cargo test`):** Panic on first violation with full context in the panic message. Includes the seed so the exact game can be replayed.

**Batch/CI (`CHESS_ORACLE_REPORT=1`):** Write JSON reports for every game to the reports directory. Still panics at the end if any violations were found, but collects all of them first across all games.

### Reproducibility

Every random game is seeded as `base_seed + game_index`. Base seed is fixed by default (deterministic CI), overridable via `CHESS_ORACLE_SEED=<u64>`.

## File Changes Summary

### New Files

| File | Purpose |
|------|---------|
| `crates/chess_core/src/testing.rs` | `MoveStrategy` trait, strategies, `GameOracle`, `InvariantChecker`, `GameRecord`, `GameReport`, reporting types |
| `crates/chess_core/tests/game_oracle.rs` | Layer 1: scripted scenarios + random game batches |
| `crates/game_app/tests/automation_game_playthrough.rs` | Layer 2: full-stack game playthroughs via `AutomationHarness` |

### Modified Files

| File | Change |
|------|--------|
| `crates/chess_core/Cargo.toml` | Add `test-support` feature, `rand`/`serde`/`serde_json` dependencies behind it |
| `crates/chess_core/src/lib.rs` | Add `pub mod testing` behind `#[cfg(feature = "test-support")]` |
| `crates/game_app/Cargo.toml` | Add `chess_core/test-support` to dev-dependencies features |
| `crates/game_app/src/automation.rs` | Add `ClaimDraw` variant to `AutomationMatchAction` |
| `crates/game_app/src/plugins/automation.rs` | Add `ClaimDraw` dispatch arm in `apply_match_action` |
| `crates/game_app/src/automation_transport.rs` | Add serde support for `ClaimDraw` (conditional on `automation-transport` feature) |

### No Changes To

- `chess_core/src/game.rs` — the rules engine is the system under test, not modified
- Existing test files — no modifications, only new files added
