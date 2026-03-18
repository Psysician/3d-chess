# AI-Driven E2E Game Testing

## Goal

Build a two-layer testing harness that plays complete chess games to find rule bugs, verify game flow integrity, and stress-test the engine. Layer 1 runs thousands of fast games at the `chess_core` domain level. Layer 2 plays a handful of complete games through the `game_app` automation harness to test the full Bevy stack.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Move selection | Scripted sequences + random fill | Scripted games cover known edge cases; random games explore states humans rarely reach |
| Reporting | Panic for `cargo test` + JSON for CI | Fast feedback in development, structured artifacts for batch analysis |
| Scale | Configurable via env var | Small default (20 games) for `cargo test`, scales to thousands for nightly/stress runs |
| Verification scope | All four: checkmate, stalemate, draw rules, move legality invariants | Complete rules coverage after every move |

## Architecture

Two layers with shared infrastructure:

```
chess_core (Layer 1 - Rules Oracle)
  ├── src/testing.rs          # MoveStrategy trait, strategies, GameOracle, InvariantChecker, reporting
  └── tests/game_oracle.rs    # Scripted scenarios + random game batches

game_app (Layer 2 - Full-Stack Playthrough)
  └── tests/automation_game_playthrough.rs   # Complete games through AutomationHarness
```

Layer 1 depends only on `chess_core`. Layer 2 depends on `game_app` (which transitively includes `chess_core`). The `MoveStrategy` trait and `InvariantChecker` live in `chess_core` so both layers share them.

## Section 1: Shared Infrastructure

### Location

`chess_core/src/testing.rs` — public module gated behind `#[cfg(feature = "test-support")]`.

### MoveStrategy Trait

```rust
pub trait MoveStrategy {
    fn select_move(&mut self, game: &GameState, legal_moves: &[Move]) -> Move;
}
```

### Built-in Strategies

**`RandomStrategy`** — Uniform random selection from legal moves. Accepts a seed for reproducibility.

**`WeightedStrategy`** — Biases toward captures (pieces on target square), checks (move puts opponent in check), promotions, en passant, and castling. Uses weighted sampling with the same seeded RNG.

**`ScriptedStrategy`** — Takes a `Vec<Move>`, plays them in order. Falls back to `RandomStrategy` when the script is exhausted. Supports patterns like "play Scholar's Mate opening, then random continuation."

### GameOracle

Plays a complete game using one strategy per side:

```rust
pub struct GameOracle {
    white: Box<dyn MoveStrategy>,
    black: Box<dyn MoveStrategy>,
}

impl GameOracle {
    pub fn play_game(&mut self, initial: GameState) -> GameRecord { ... }
}
```

Games are capped at 500 moves by default (configurable) to prevent infinite random-play loops.

### GameRecord

```rust
pub struct GameRecord {
    pub initial_fen: String,
    pub moves: Vec<Move>,
    pub final_fen: String,
    pub outcome: GameStatus,
    pub violations: Vec<Violation>,
    pub move_count: u16,
}
```

### InvariantChecker

Runs after every `apply_move` call. Checks:

1. **King safety** — The side that just moved does not have its king in check (the engine should prevent this, but we verify).
2. **King count** — Exactly one king per side at all times.
3. **Piece count consistency** — No pieces appear or disappear except through captures and promotions. Total piece count only decreases (captures) or stays the same (quiet moves, promotions replace a pawn with another piece).
4. **Castling rights monotonicity** — Castling rights only decrease, never increase.
5. **En passant validity** — If en passant target is set, it is on the correct rank (rank 3 for Black's target, rank 6 for White's target) and a pawn of the correct side is on the adjacent rank.
6. **Halfmove clock** — Resets to 0 on pawn moves and captures, increments by 1 otherwise.
7. **Position history** — Tracks correctly for repetition detection.

### New Dependencies

- `rand` added to `chess_core` as a dev-dependency (for strategies in tests).
- `serde` and `serde_json` already available in the workspace for JSON reporting.

The `test-support` feature in `chess_core/Cargo.toml` enables the `testing` module and adds `rand` + `serde`/`serde_json` as dependencies (not dev-dependencies, since downstream crates like `game_app` need to use the module in their tests too).

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

### Scenarios

| Scenario | What it tests |
|----------|---------------|
| Complete game to checkmate | `StartNewMatch` -> scripted Scholar's Mate via `SubmitMove` -> verify screen transitions to `MatchResult` -> verify snapshot reports correct winner |
| Mid-game save/load | Play 5 moves -> `SaveManual` -> `ReturnToMenu` -> load the save -> verify FEN matches -> continue to completion |
| Recovery resume | Play 3 moves -> drop harness (simulating crash) -> new harness boots -> verify recovery banner -> resume -> verify FEN matches -> play to completion |
| Promotion through automation | Scripted promotion position -> `SubmitMove` with promotion piece -> verify piece type in snapshot |
| Draw claim flow | Play to claimable draw position -> verify `draw_available` in snapshot -> claim draw -> verify `MatchResult` with draw outcome |
| Rematch after completion | Play to checkmate -> `Rematch` -> verify fresh starting position -> play a few moves |
| Stalemate through full stack | Scripted stalemate -> verify `GameOutcome::Draw(Stalemate)` in snapshot |

### Scale

Fixed at 7 scenarios. These are deterministic integration tests, not fuzzing. They exercise the automation command pipeline, snapshot accuracy, persistence round-trips, and screen state transitions.

## Section 4: JSON Reporting

### Report Structure

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
| `crates/chess_core/src/testing.rs` | `MoveStrategy` trait, strategies, `GameOracle`, `InvariantChecker`, `GameRecord`, reporting types |
| `crates/chess_core/tests/game_oracle.rs` | Layer 1: scripted scenarios + random game batches |
| `crates/game_app/tests/automation_game_playthrough.rs` | Layer 2: full-stack game playthroughs via `AutomationHarness` |

### Modified Files

| File | Change |
|------|--------|
| `crates/chess_core/Cargo.toml` | Add `test-support` feature, `rand`/`serde`/`serde_json` dependencies behind it |
| `crates/chess_core/src/lib.rs` | Add `pub mod testing` behind `#[cfg(feature = "test-support")]` |
| `crates/game_app/Cargo.toml` | Add `chess_core/test-support` to dev-dependencies features |

### No Changes To

- `chess_core/src/game.rs` — the rules engine is the system under test, not modified
- `game_app/src/automation.rs` — the automation seam is used as-is
- Existing test files — no modifications, only new files added
