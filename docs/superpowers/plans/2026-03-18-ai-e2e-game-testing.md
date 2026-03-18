# AI-Driven E2E Game Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a two-layer testing harness that plays complete chess games to verify rules correctness, game flow integrity, and stress resilience.

**Architecture:** Layer 1 (chess_core) runs high-volume game simulations at the domain level with invariant checking after every move. Layer 2 (game_app) plays complete games through the AutomationHarness to test the full Bevy stack. Shared infrastructure (strategies, invariant checker, reporting) lives in a feature-gated `testing` module in chess_core.

**Tech Stack:** Rust, chess_core domain types, rand (seeded RNG), serde_json (reporting), Bevy headless (Layer 2)

**Spec:** `docs/superpowers/specs/2026-03-18-ai-e2e-game-testing-design.md`

---

### Task 1: Add `test-support` Feature and Testing Module Scaffold

**Files:**
- Modify: `crates/chess_core/Cargo.toml`
- Modify: `crates/chess_core/src/lib.rs`
- Create: `crates/chess_core/src/testing.rs`

- [ ] **Step 1: Add `test-support` feature to chess_core Cargo.toml**

In `crates/chess_core/Cargo.toml`, add a `[features]` section and the `rand` dependency:

```toml
[features]
default = []
test-support = ["dep:rand", "dep:serde_json"]

[dependencies]
serde.workspace = true
rand = { version = "0.9", optional = true }
serde_json = { workspace = true, optional = true }

[dev-dependencies]
chess_core = { path = ".", features = ["test-support"] }
serde_json.workspace = true
```

Note: `serde` is already a non-optional dependency (always available). The feature only adds `rand` and `serde_json` as optional deps. The `dev-dependencies` self-reference enables the feature for chess_core's own integration tests.

- [ ] **Step 2: Add conditional module to lib.rs**

In `crates/chess_core/src/lib.rs`, after the existing `pub use` block, add:

```rust
#[cfg(feature = "test-support")]
pub mod testing;
```

- [ ] **Step 3: Create empty testing module**

Create `crates/chess_core/src/testing.rs` with a module-level doc comment:

```rust
//! Test infrastructure for playing complete games and verifying invariants.
//!
//! Gated behind the `test-support` feature because downstream crates (e.g. `game_app`)
//! need these types in their integration tests. `cfg(test)` would not work because it
//! only applies when compiling the crate itself for testing.
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p chess_core --features test-support`
Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/chess_core/Cargo.toml crates/chess_core/src/lib.rs crates/chess_core/src/testing.rs
git commit -m "Add test-support feature and empty testing module scaffold"
```

---

### Task 2: Implement MoveStrategy Trait and Built-in Strategies

**Files:**
- Modify: `crates/chess_core/src/testing.rs`

- [ ] **Step 1: Write a test for RandomStrategy**

Add to the bottom of `crates/chess_core/src/testing.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameState, Square};

    #[test]
    fn random_strategy_selects_from_legal_moves() {
        let game = GameState::starting_position();
        let legal_moves = game.legal_moves();
        let mut strategy = RandomStrategy::new(42);
        let selected = strategy.select_move(&game, &legal_moves);
        assert!(
            legal_moves.contains(&selected),
            "random strategy must return a move from the legal moves list"
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p chess_core --features test-support -- tests::random_strategy_selects_from_legal_moves`
Expected: FAIL — `RandomStrategy` not found.

- [ ] **Step 3: Implement MoveStrategy trait and RandomStrategy**

Add to the top of `crates/chess_core/src/testing.rs` (after the doc comment):

```rust
use rand::prelude::*;
use rand::rngs::StdRng;

use crate::{GameState, Move};

/// Strategy for selecting moves during automated game play.
pub trait MoveStrategy {
    fn select_move(&mut self, game: &GameState, legal_moves: &[Move]) -> Move;
}

/// Uniform random selection from legal moves.
pub struct RandomStrategy {
    rng: StdRng,
}

impl RandomStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl MoveStrategy for RandomStrategy {
    fn select_move(&mut self, _game: &GameState, legal_moves: &[Move]) -> Move {
        *legal_moves
            .choose(&mut self.rng)
            .expect("legal_moves must be non-empty when calling select_move")
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p chess_core --features test-support -- tests::random_strategy_selects_from_legal_moves`
Expected: PASS

- [ ] **Step 5: Write a test for WeightedStrategy**

Add to the `tests` module:

```rust
    #[test]
    fn weighted_strategy_selects_from_legal_moves() {
        let game = GameState::starting_position();
        let legal_moves = game.legal_moves();
        let mut strategy = WeightedStrategy::new(42);
        let selected = strategy.select_move(&game, &legal_moves);
        assert!(
            legal_moves.contains(&selected),
            "weighted strategy must return a move from the legal moves list"
        );
    }

    #[test]
    fn weighted_strategy_biases_toward_captures() {
        // Position where white can capture or make a quiet move.
        // White queen on d4 can capture black pawn on d7 or move to many quiet squares.
        let game = GameState::from_fen("4k3/3p4/8/8/3Q4/8/8/4K3 w - - 0 1")
            .expect("test FEN should parse");
        let legal_moves = game.legal_moves();
        let capture = Move::new(
            Square::from_algebraic("d4").expect("valid"),
            Square::from_algebraic("d7").expect("valid"),
        );
        assert!(legal_moves.contains(&capture), "capture must be in legal moves");

        // Run 100 trials and count how many times the capture is selected.
        let mut capture_count = 0;
        for seed in 0..100 {
            let mut strategy = WeightedStrategy::new(seed);
            if strategy.select_move(&game, &legal_moves) == capture {
                capture_count += 1;
            }
        }
        // With 3x weight for captures vs 1x for quiet moves, capture rate should be
        // significantly higher than uniform (1/N). Don't assert exact rate — just that
        // bias exists (> 5% compared to ~3.7% uniform with 27 legal moves).
        assert!(
            capture_count > 5,
            "weighted strategy should select captures more often than uniform, got {capture_count}/100"
        );
    }
```

- [ ] **Step 6: Implement WeightedStrategy**

Add after `RandomStrategy` in `testing.rs`:

```rust
/// Biased selection that favors captures, promotions, en passant, and castling.
///
/// Weights (not configurable — change these constants to tune):
/// - Capture: 3x
/// - Promotion: 5x
/// - En passant: 4x
/// - Castling: 2x
/// - Quiet move: 1x
pub struct WeightedStrategy {
    rng: StdRng,
}

impl WeightedStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn weight_for(game: &GameState, mv: &Move) -> u32 {
        let is_capture = game.piece_at(mv.to()).is_some();
        let is_promotion = mv.promotion().is_some();
        let is_en_passant = game.en_passant_target() == Some(mv.to())
            && game
                .piece_at(mv.from())
                .is_some_and(|p| p.kind == crate::PieceKind::Pawn);
        let is_castling = game
            .piece_at(mv.from())
            .is_some_and(|p| p.kind == crate::PieceKind::King)
            && mv.from().file().abs_diff(mv.to().file()) == 2;

        if is_promotion {
            5
        } else if is_en_passant {
            4
        } else if is_capture {
            3
        } else if is_castling {
            2
        } else {
            1
        }
    }
}

impl MoveStrategy for WeightedStrategy {
    fn select_move(&mut self, game: &GameState, legal_moves: &[Move]) -> Move {
        let weights: Vec<u32> = legal_moves.iter().map(|mv| Self::weight_for(game, mv)).collect();
        let total: u32 = weights.iter().sum();
        let mut roll = self.rng.random_range(0..total);
        for (i, &w) in weights.iter().enumerate() {
            if roll < w {
                return legal_moves[i];
            }
            roll -= w;
        }
        // Fallback (should never reach here if weights are non-zero).
        legal_moves[legal_moves.len() - 1]
    }
}
```

- [ ] **Step 7: Run both tests to verify they pass**

Run: `cargo test -p chess_core --features test-support -- tests::weighted_strategy`
Expected: Both PASS.

- [ ] **Step 8: Write a test for ScriptedStrategy**

Add to the `tests` module:

```rust
    #[test]
    fn scripted_strategy_plays_script_then_falls_back_to_random() {
        let game = GameState::starting_position();
        let legal_moves = game.legal_moves();

        let e2e4 = Move::new(
            Square::from_algebraic("e2").expect("valid"),
            Square::from_algebraic("e4").expect("valid"),
        );
        let mut strategy = ScriptedStrategy::new(vec![e2e4], 99);

        // First call: returns scripted move.
        let first = strategy.select_move(&game, &legal_moves);
        assert_eq!(first, e2e4, "first move should be the scripted one");

        // Second call: script exhausted, falls back to random.
        let second = strategy.select_move(&game, &legal_moves);
        assert!(
            legal_moves.contains(&second),
            "fallback move must be from legal moves"
        );
    }
```

- [ ] **Step 9: Implement ScriptedStrategy**

Add after `WeightedStrategy` in `testing.rs`:

```rust
/// Plays scripted moves in order, then falls back to random selection.
pub struct ScriptedStrategy {
    script: Vec<Move>,
    index: usize,
    fallback: RandomStrategy,
}

impl ScriptedStrategy {
    pub fn new(script: Vec<Move>, fallback_seed: u64) -> Self {
        Self {
            script,
            index: 0,
            fallback: RandomStrategy::new(fallback_seed),
        }
    }
}

impl MoveStrategy for ScriptedStrategy {
    fn select_move(&mut self, game: &GameState, legal_moves: &[Move]) -> Move {
        while self.index < self.script.len() {
            let scripted = self.script[self.index];
            self.index += 1;
            if legal_moves.contains(&scripted) {
                return scripted;
            }
            // Scripted move not legal in this position — skip it and try next.
        }
        self.fallback.select_move(game, legal_moves)
    }
}
```

- [ ] **Step 10: Run all strategy tests**

Run: `cargo test -p chess_core --features test-support -- tests::`
Expected: All 4 tests PASS.

- [ ] **Step 11: Commit**

```bash
git add crates/chess_core/src/testing.rs
git commit -m "Implement MoveStrategy trait with random, weighted, and scripted strategies"
```

---

### Task 3: Implement InvariantChecker

**Files:**
- Modify: `crates/chess_core/src/testing.rs`

- [ ] **Step 1: Write tests for individual invariants**

Add to the `tests` module in `testing.rs`:

```rust
    #[test]
    fn invariant_checker_passes_for_a_legal_opening_move() {
        let before = GameState::starting_position();
        let mv = Move::new(
            Square::from_algebraic("e2").expect("valid"),
            Square::from_algebraic("e4").expect("valid"),
        );
        let after = before.apply_move(mv).expect("legal move");
        let violations = InvariantChecker::check(&before, &mv, &after);
        assert!(violations.is_empty(), "no violations expected for legal e2e4, got: {violations:?}");
    }

    #[test]
    fn invariant_checker_detects_fullmove_number_increment_after_black() {
        // After black moves, fullmove number should increment.
        let after_e4 = GameState::starting_position()
            .apply_move(Move::new(
                Square::from_algebraic("e2").expect("valid"),
                Square::from_algebraic("e4").expect("valid"),
            ))
            .expect("legal");
        assert_eq!(after_e4.fullmove_number(), 1, "fullmove stays 1 after white moves");

        let mv = Move::new(
            Square::from_algebraic("e7").expect("valid"),
            Square::from_algebraic("e5").expect("valid"),
        );
        let after_e5 = after_e4.apply_move(mv).expect("legal");
        assert_eq!(after_e5.fullmove_number(), 2, "fullmove becomes 2 after black moves");

        let violations = InvariantChecker::check(&after_e4, &mv, &after_e5);
        assert!(violations.is_empty(), "no violations for legal e7e5, got: {violations:?}");
    }

    #[test]
    fn invariant_checker_validates_capture_piece_count() {
        // Set up a position where white captures a black piece.
        let game = GameState::from_fen("4k3/3p4/8/8/3Q4/8/8/4K3 w - - 0 1")
            .expect("test FEN should parse");
        let capture = Move::new(
            Square::from_algebraic("d4").expect("valid"),
            Square::from_algebraic("d7").expect("valid"),
        );
        let after = game.apply_move(capture).expect("legal capture");
        let violations = InvariantChecker::check(&game, &capture, &after);
        assert!(violations.is_empty(), "no violations for legal capture, got: {violations:?}");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p chess_core --features test-support -- tests::invariant_checker`
Expected: FAIL — `InvariantChecker` not found.

- [ ] **Step 3: Implement Violation type and InvariantChecker**

Add after `ScriptedStrategy` in `testing.rs`:

```rust
use std::collections::HashMap;

use crate::{BoardState, CastlingRights, Piece, PieceKind, Side, Square};

/// A rule violation detected by the invariant checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    KingInCheck,
    KingCountInvalid,
    PieceCountInconsistent,
    CastlingRightsIncreased,
    EnPassantInvalid,
    HalfmoveClockWrong,
    FullmoveNumberWrong,
    PositionHistoryInconsistent,
    LegalMoveRejected,
}

fn count_pieces(board: &BoardState) -> std::collections::HashMap<(Side, PieceKind), u8> {
    let mut counts = std::collections::HashMap::new();
    for (_sq, piece) in board.iter() {
        *counts.entry((piece.side, piece.kind)).or_insert(0) += 1;
    }
    counts
}

fn castling_rights_as_tuple(cr: CastlingRights) -> (bool, bool, bool, bool) {
    (
        cr.kingside(Side::White),
        cr.queenside(Side::White),
        cr.kingside(Side::Black),
        cr.queenside(Side::Black),
    )
}

/// Post-move invariant checker. Call after every `apply_move`.
pub struct InvariantChecker;

impl InvariantChecker {
    pub fn check(before: &GameState, mv: &Move, after: &GameState) -> Vec<Violation> {
        let mut violations = Vec::new();

        Self::check_king_safety(after, before.side_to_move(), &mut violations);
        Self::check_king_count(after, &mut violations);
        Self::check_piece_counts(before, mv, after, &mut violations);
        Self::check_castling_monotonicity(before, after, &mut violations);
        Self::check_en_passant(after, &mut violations);
        Self::check_halfmove_clock(before, mv, after, &mut violations);
        Self::check_fullmove_number(before, after, &mut violations);
        Self::check_position_history(before, after, &mut violations);

        violations
    }

    fn check_king_safety(after: &GameState, moved_side: Side, violations: &mut Vec<Violation>) {
        // The side that just moved should not have its king in check.
        // GameState::is_in_check(side) is public and checks if `side`'s king is attacked.
        if after.board().king_square(moved_side).is_none() {
            violations.push(Violation {
                kind: ViolationKind::KingInCheck,
                description: format!("{moved_side:?} king not found after move"),
            });
            return;
        }
        if after.is_in_check(moved_side) {
            violations.push(Violation {
                kind: ViolationKind::KingInCheck,
                description: format!("{moved_side:?} king is in check after their own move"),
            });
        }
    }

    fn check_king_count(after: &GameState, violations: &mut Vec<Violation>) {
        for side in [Side::White, Side::Black] {
            let king_count = after
                .board()
                .iter()
                .filter(|(_, p)| p.side == side && p.kind == PieceKind::King)
                .count();
            if king_count != 1 {
                violations.push(Violation {
                    kind: ViolationKind::KingCountInvalid,
                    description: format!("{side:?} has {king_count} kings, expected 1"),
                });
            }
        }
    }

    fn check_piece_counts(
        before: &GameState,
        mv: &Move,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let before_counts = count_pieces(before.board());
        let after_counts = count_pieces(after.board());
        let before_total: u8 = before_counts.values().sum();
        let after_total: u8 = after_counts.values().sum();

        let moved_piece = before
            .piece_at(mv.from())
            .expect("moved piece must exist at source");
        let is_capture = before.piece_at(mv.to()).is_some();
        let is_en_passant = moved_piece.kind == PieceKind::Pawn
            && before.en_passant_target() == Some(mv.to())
            && !is_capture;
        let is_promotion = mv.promotion().is_some();

        if is_capture || is_en_passant {
            if after_total != before_total - 1 {
                violations.push(Violation {
                    kind: ViolationKind::PieceCountInconsistent,
                    description: format!(
                        "capture should reduce total pieces by 1: before={before_total}, after={after_total}"
                    ),
                });
            }
        } else if is_promotion {
            // Promotion without capture: total stays the same (pawn becomes another piece).
            if after_total != before_total {
                violations.push(Violation {
                    kind: ViolationKind::PieceCountInconsistent,
                    description: format!(
                        "promotion (no capture) should keep total pieces: before={before_total}, after={after_total}"
                    ),
                });
            }
        } else {
            // Quiet move: total unchanged.
            if after_total != before_total {
                violations.push(Violation {
                    kind: ViolationKind::PieceCountInconsistent,
                    description: format!(
                        "quiet move should keep total pieces: before={before_total}, after={after_total}"
                    ),
                });
            }
        }

        // Total never increases.
        if after_total > before_total {
            violations.push(Violation {
                kind: ViolationKind::PieceCountInconsistent,
                description: format!(
                    "total piece count must never increase: before={before_total}, after={after_total}"
                ),
            });
        }
    }

    fn check_castling_monotonicity(
        before: &GameState,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let b = castling_rights_as_tuple(before.castling_rights());
        let a = castling_rights_as_tuple(after.castling_rights());
        // Each right can only go from true to false, never false to true.
        if (!b.0 && a.0) || (!b.1 && a.1) || (!b.2 && a.2) || (!b.3 && a.3) {
            violations.push(Violation {
                kind: ViolationKind::CastlingRightsIncreased,
                description: format!(
                    "castling rights increased: before={b:?}, after={a:?}"
                ),
            });
        }
    }

    fn check_en_passant(after: &GameState, violations: &mut Vec<Violation>) {
        if let Some(target) = after.en_passant_target() {
            let rank = target.rank();
            // 0-indexed: rank 2 for Black's target (white pawn just double-pushed to rank 3),
            // rank 5 for White's target (black pawn just double-pushed to rank 4).
            if rank != 2 && rank != 5 {
                violations.push(Violation {
                    kind: ViolationKind::EnPassantInvalid,
                    description: format!(
                        "en passant target {target} has rank {rank}, expected 2 or 5"
                    ),
                });
            }
        }
    }

    fn check_halfmove_clock(
        before: &GameState,
        mv: &Move,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let moved_piece = before
            .piece_at(mv.from())
            .expect("moved piece must exist at source");
        let is_capture = before.piece_at(mv.to()).is_some()
            || (moved_piece.kind == PieceKind::Pawn
                && before.en_passant_target() == Some(mv.to()));
        let is_pawn_move = moved_piece.kind == PieceKind::Pawn;

        let expected = if is_pawn_move || is_capture {
            0
        } else {
            before.halfmove_clock() + 1
        };

        if after.halfmove_clock() != expected {
            violations.push(Violation {
                kind: ViolationKind::HalfmoveClockWrong,
                description: format!(
                    "halfmove clock: expected {expected}, got {}",
                    after.halfmove_clock()
                ),
            });
        }
    }

    fn check_fullmove_number(
        before: &GameState,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let expected = if before.side_to_move() == Side::Black {
            before.fullmove_number() + 1
        } else {
            before.fullmove_number()
        };

        if after.fullmove_number() != expected {
            violations.push(Violation {
                kind: ViolationKind::FullmoveNumberWrong,
                description: format!(
                    "fullmove number: expected {expected}, got {} (moved side: {:?})",
                    after.fullmove_number(),
                    before.side_to_move()
                ),
            });
        }
    }

    fn check_position_history(
        before: &GameState,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let expected_len = before.position_history().len() + 1;
        if after.position_history().len() != expected_len {
            violations.push(Violation {
                kind: ViolationKind::PositionHistoryInconsistent,
                description: format!(
                    "position history length: expected {expected_len}, got {}",
                    after.position_history().len()
                ),
            });
        }
    }
}
```

- [ ] **Step 4: Run all invariant checker tests**

Run: `cargo test -p chess_core --features test-support -- tests::invariant_checker`
Expected: All 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/chess_core/src/testing.rs
git commit -m "Implement InvariantChecker with 8 post-move invariant checks"
```

---

### Task 4: Implement GameOracle and GameRecord

**Files:**
- Modify: `crates/chess_core/src/testing.rs`

- [ ] **Step 1: Write test for GameOracle completing a scripted checkmate**

Add to the `tests` module:

```rust
    use crate::{GameOutcome, GameStatus, PieceKind, WinReason};

    #[test]
    fn game_oracle_plays_scholars_mate_to_checkmate() {
        let script_white = vec![
            Move::new(sq("e2"), sq("e4")),
            Move::new(sq("d1"), sq("h5")),
            Move::new(sq("f1"), sq("c4")),
            Move::new(sq("h5"), sq("f7")),
        ];
        let script_black = vec![
            Move::new(sq("e7"), sq("e5")),
            Move::new(sq("b8"), sq("c6")),
            Move::new(sq("g8"), sq("f6")),
        ];

        let mut oracle = GameOracle::new(
            Box::new(ScriptedStrategy::new(script_white, 0)),
            Box::new(ScriptedStrategy::new(script_black, 0)),
        );
        let record = oracle.play_game(GameState::starting_position());

        assert!(record.violations.is_empty(), "no violations: {:?}", record.violations);
        assert_eq!(record.move_count, 7);
        match record.termination {
            GameTermination::Completed(GameStatus::Finished(GameOutcome::Win {
                winner: Side::White,
                reason: WinReason::Checkmate,
            })) => {}
            other => panic!("expected White checkmate, got {other:?}"),
        }
    }

    fn sq(name: &str) -> Square {
        Square::from_algebraic(name).expect("test square must be valid")
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p chess_core --features test-support -- tests::game_oracle_plays_scholars_mate`
Expected: FAIL — `GameOracle` not found.

- [ ] **Step 3: Implement GameTermination, GameRecord, and GameOracle**

Add after `InvariantChecker` in `testing.rs`:

```rust
use crate::{GameStatus, MoveError};

/// How a game ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTermination {
    /// Normal game end (checkmate, stalemate, draw).
    Completed(GameStatus),
    /// Game capped at max_moves without reaching a terminal position.
    MoveLimitReached(u16),
}

/// Full record of a played game.
#[derive(Debug, Clone)]
pub struct GameRecord {
    pub initial_fen: String,
    pub moves: Vec<Move>,
    pub final_fen: String,
    pub termination: GameTermination,
    pub violations: Vec<Violation>,
    pub move_count: u16,
}

/// Plays complete games using a strategy per side.
pub struct GameOracle {
    white: Box<dyn MoveStrategy>,
    black: Box<dyn MoveStrategy>,
    max_moves: u16,
}

impl GameOracle {
    pub fn new(white: Box<dyn MoveStrategy>, black: Box<dyn MoveStrategy>) -> Self {
        Self {
            white,
            black,
            max_moves: 500,
        }
    }

    pub fn with_max_moves(mut self, max_moves: u16) -> Self {
        self.max_moves = max_moves;
        self
    }

    pub fn play_game(&mut self, initial: GameState) -> GameRecord {
        let initial_fen = initial.to_fen();
        let mut game = initial;
        let mut moves = Vec::new();
        let mut violations = Vec::new();
        let mut move_count: u16 = 0;

        loop {
            let status = game.status();
            if status.is_finished() {
                return GameRecord {
                    initial_fen,
                    moves,
                    final_fen: game.to_fen(),
                    termination: GameTermination::Completed(status),
                    violations,
                    move_count,
                };
            }

            if move_count >= self.max_moves {
                return GameRecord {
                    initial_fen,
                    moves,
                    final_fen: game.to_fen(),
                    termination: GameTermination::MoveLimitReached(self.max_moves),
                    violations,
                    move_count,
                };
            }

            let legal_moves = game.legal_moves();
            if legal_moves.is_empty() {
                // status() should have caught this — this is itself a violation.
                violations.push(Violation {
                    kind: ViolationKind::PositionHistoryInconsistent,
                    description: format!(
                        "legal_moves() is empty but status() returned {status:?} at move {move_count}"
                    ),
                });
                return GameRecord {
                    initial_fen,
                    moves,
                    final_fen: game.to_fen(),
                    termination: GameTermination::Completed(status),
                    violations,
                    move_count,
                };
            }

            let strategy: &mut dyn MoveStrategy = match game.side_to_move() {
                Side::White => self.white.as_mut(),
                Side::Black => self.black.as_mut(),
            };
            let chosen = strategy.select_move(&game, &legal_moves);

            match game.apply_move(chosen) {
                Ok(next) => {
                    let move_violations = InvariantChecker::check(&game, &chosen, &next);
                    violations.extend(move_violations);
                    moves.push(chosen);
                    game = next;
                    move_count += 1;
                }
                Err(error) => {
                    violations.push(Violation {
                        kind: ViolationKind::LegalMoveRejected,
                        description: format!(
                            "apply_move rejected a move from legal_moves(): {chosen} — {error}"
                        ),
                    });
                    // Try to continue with a different move.
                    let fallback = legal_moves
                        .iter()
                        .find(|m| **m != chosen && game.apply_move(**m).is_ok());
                    if let Some(&fallback_mv) = fallback {
                        let next = game
                            .apply_move(fallback_mv)
                            .expect("fallback from legal_moves must succeed");
                        moves.push(fallback_mv);
                        game = next;
                        move_count += 1;
                    } else {
                        return GameRecord {
                            initial_fen,
                            moves,
                            final_fen: game.to_fen(),
                            termination: GameTermination::Completed(game.status()),
                            violations,
                            move_count,
                        };
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p chess_core --features test-support -- tests::game_oracle_plays_scholars_mate`
Expected: PASS

- [ ] **Step 5: Write test for move limit**

Add to tests module:

```rust
    #[test]
    fn game_oracle_respects_move_limit() {
        let mut oracle = GameOracle::new(
            Box::new(RandomStrategy::new(42)),
            Box::new(RandomStrategy::new(99)),
        )
        .with_max_moves(10);

        let record = oracle.play_game(GameState::starting_position());
        match record.termination {
            GameTermination::MoveLimitReached(10) => {}
            GameTermination::Completed(_) => {
                // Game ended naturally before 10 moves — that's fine too.
            }
            other => panic!("unexpected termination: {other:?}"),
        }
        assert!(record.move_count <= 10);
    }
```

- [ ] **Step 6: Run test and verify pass**

Run: `cargo test -p chess_core --features test-support -- tests::game_oracle_respects_move_limit`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/chess_core/src/testing.rs
git commit -m "Implement GameOracle with game loop, move limit, and violation tracking"
```

---

### Task 5: Implement JSON Reporting (GameReport)

**Files:**
- Modify: `crates/chess_core/src/testing.rs`

- [ ] **Step 1: Write test for GameReport serialization**

Add to tests module:

```rust
    #[test]
    fn game_report_serializes_to_json() {
        let record = GameRecord {
            initial_fen: String::from("startpos"),
            moves: vec![Move::new(sq("e2"), sq("e4"))],
            final_fen: String::from("after"),
            termination: GameTermination::MoveLimitReached(1),
            violations: vec![],
            move_count: 1,
        };
        let report = GameReport::from_record(&record, 42, "random");
        let json = serde_json::to_string_pretty(&report).expect("should serialize");
        assert!(json.contains("\"seed\": 42"));
        assert!(json.contains("\"strategy\": \"random\""));
        assert!(json.contains("\"total_moves\": 1"));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p chess_core --features test-support -- tests::game_report_serializes`
Expected: FAIL — `GameReport` not found.

- [ ] **Step 3: Implement GameReport and related types**

Add after `GameRecord` in `testing.rs`:

```rust
use serde::Serialize;

/// JSON-serializable game report for CI artifact collection.
#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct MoveRecord {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViolationRecord {
    pub move_number: u16,
    pub fen_before: String,
    pub attempted_move: String,
    pub violation: String,
}

impl GameReport {
    pub fn from_record(record: &GameRecord, seed: u64, strategy: &str) -> Self {
        let outcome = match &record.termination {
            GameTermination::Completed(status) => format!("{status:?}"),
            GameTermination::MoveLimitReached(limit) => format!("MoveLimitReached({limit})"),
        };

        let moves = record
            .moves
            .iter()
            .map(|mv| MoveRecord {
                from: mv.from().to_algebraic(),
                to: mv.to().to_algebraic(),
                promotion: mv.promotion().map(|p| p.fen_letter().to_string()),
            })
            .collect();

        let violations = record
            .violations
            .iter()
            .enumerate()
            .map(|(i, v)| ViolationRecord {
                move_number: u16::try_from(i).unwrap_or(u16::MAX),
                fen_before: String::new(),
                attempted_move: String::new(),
                violation: v.description.clone(),
            })
            .collect();

        Self {
            seed,
            initial_fen: record.initial_fen.clone(),
            moves,
            outcome,
            violations,
            total_moves: record.move_count,
            strategy: strategy.to_string(),
            timestamp_utc: String::from("test"),
        }
    }

    /// Writes the report to a JSON file in the given directory.
    pub fn write_to_dir(&self, dir: &std::path::Path, game_index: usize) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("game_{game_index:04}.json"));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p chess_core --features test-support -- tests::game_report_serializes`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/chess_core/src/testing.rs
git commit -m "Implement GameReport for JSON-serializable test output"
```

---

### Task 6: Layer 1 — Scripted Scenario Tests

**Files:**
- Create: `crates/chess_core/tests/game_oracle.rs`

- [ ] **Step 1: Create the test file with helpers and first scripted scenario (Scholar's Mate)**

Create `crates/chess_core/tests/game_oracle.rs`:

```rust
use chess_core::testing::{
    GameOracle, GameRecord, GameTermination, RandomStrategy, ScriptedStrategy, WeightedStrategy,
};
use chess_core::{
    DrawReason, GameOutcome, GameState, GameStatus, Move, PieceKind, Side, Square, WinReason,
};

fn sq(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn assert_checkmate(record: &GameRecord, winner: Side) {
    assert!(
        record.violations.is_empty(),
        "violations: {:?}",
        record.violations
    );
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Win {
            winner: w,
            reason: WinReason::Checkmate,
        })) if *w == winner => {}
        other => panic!("expected {winner:?} checkmate, got {other:?}"),
    }
}

fn assert_draw(record: &GameRecord, expected_reason: DrawReason) {
    assert!(
        record.violations.is_empty(),
        "violations: {:?}",
        record.violations
    );
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Draw(reason)))
            if *reason == expected_reason => {}
        other => panic!("expected draw {expected_reason:?}, got {other:?}"),
    }
}

fn assert_no_violations(record: &GameRecord) {
    assert!(
        record.violations.is_empty(),
        "violations: {:?}",
        record.violations
    );
}

// --- Scripted Scenarios ---

#[test]
fn scholars_mate_reaches_checkmate_in_7_ply() {
    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("e2"), sq("e4")),
                Move::new(sq("d1"), sq("h5")),
                Move::new(sq("f1"), sq("c4")),
                Move::new(sq("h5"), sq("f7")),
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("e7"), sq("e5")),
                Move::new(sq("b8"), sq("c6")),
                Move::new(sq("g8"), sq("f6")),
            ],
            0,
        )),
    );
    let record = oracle.play_game(GameState::starting_position());
    assert_checkmate(&record, Side::White);
    assert_eq!(record.move_count, 7);
}

#[test]
fn fools_mate_reaches_checkmate_in_4_ply() {
    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("f2"), sq("f3")),
                Move::new(sq("g2"), sq("g4")),
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("e7"), sq("e5")),
                Move::new(sq("d8"), sq("h4")),
            ],
            0,
        )),
    );
    let record = oracle.play_game(GameState::starting_position());
    assert_checkmate(&record, Side::Black);
    assert_eq!(record.move_count, 4);
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p chess_core --features test-support --test game_oracle -- scholars_mate fools_mate`
Expected: Both PASS.

- [ ] **Step 3: Add en passant, castling, promotion, and stalemate scenarios**

Append to `game_oracle.rs`:

```rust
#[test]
fn en_passant_capture_completes_without_violations() {
    // White plays e2-e4, d2-d4; Black plays d7-d5; White plays e4-e5; Black plays f7-f5;
    // White plays e5xf6 (en passant)
    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("e2"), sq("e4")),
                Move::new(sq("e4"), sq("e5")),
                Move::new(sq("e5"), sq("f6")), // en passant
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(sq("d7"), sq("d5")),
                Move::new(sq("f7"), sq("f5")), // triggers en passant
            ],
            0,
        )),
    )
    .with_max_moves(6);
    let record = oracle.play_game(GameState::starting_position());
    assert_no_violations(&record);
    assert!(record.move_count >= 5, "should play at least the scripted moves");
}

#[test]
fn castling_both_sides_completes_without_violations() {
    // Use a FEN where both sides can castle immediately.
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let game = GameState::from_fen(fen).expect("test FEN should parse");
    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(
            vec![Move::new(sq("e1"), sq("g1"))], // white kingside castle
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![Move::new(sq("e8"), sq("c8"))], // black queenside castle
            0,
        )),
    )
    .with_max_moves(4);
    let record = oracle.play_game(game);
    assert_no_violations(&record);
    assert!(record.move_count >= 2);
}

#[test]
fn promotion_to_all_four_piece_types_without_violations() {
    for piece in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
        let game = GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1")
            .expect("test FEN should parse");
        let mut oracle = GameOracle::new(
            Box::new(ScriptedStrategy::new(
                vec![Move::with_promotion(sq("a7"), sq("a8"), piece)],
                0,
            )),
            Box::new(RandomStrategy::new(42)),
        )
        .with_max_moves(2);
        let record = oracle.play_game(game);
        assert_no_violations(&record);
    }
}

#[test]
fn stalemate_position_detects_draw() {
    let game = GameState::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")
        .expect("test FEN should parse");
    let mut oracle = GameOracle::new(
        Box::new(RandomStrategy::new(0)),
        Box::new(RandomStrategy::new(0)),
    );
    let record = oracle.play_game(game);
    assert_draw(&record, DrawReason::Stalemate);
    assert_eq!(record.move_count, 0);
}

#[test]
fn threefold_repetition_detected_via_knight_shuttle() {
    let start = GameState::from_fen("4k3/8/8/8/8/8/N7/4K3 w - - 0 1")
        .expect("test FEN should parse");
    // Two full cycles = 8 moves. After two more cycles (4 more cycles total = 16 moves)
    // the engine triggers 5-fold automatic draw.
    let cycle_white = [Move::new(sq("a2"), sq("b4")), Move::new(sq("b4"), sq("a2"))];
    let cycle_black = [Move::new(sq("e8"), sq("d8")), Move::new(sq("d8"), sq("e8"))];
    let mut script_w = Vec::new();
    let mut script_b = Vec::new();
    for _ in 0..5 {
        script_w.extend_from_slice(&cycle_white);
        script_b.extend_from_slice(&cycle_black);
    }

    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(script_w, 0)),
        Box::new(ScriptedStrategy::new(script_b, 0)),
    );
    let record = oracle.play_game(start);
    assert_no_violations(&record);
    // Should terminate as fivefold repetition (automatic draw).
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Draw(_))) => {}
        other => panic!("expected draw termination, got {other:?}"),
    }
}

#[test]
fn en_passant_exposing_king_is_not_played() {
    // Position where en passant would expose the king to a rook on the same rank.
    // White pawn on e5, black pawn just played d7-d5. En passant e5xd6 would expose
    // white king on e1 to black rook on e8 (through the e-file after pawn leaves).
    let game = GameState::from_fen("4r1k1/8/8/3pP3/8/8/8/4K3 w - d6 0 1")
        .expect("test FEN should parse");
    let en_passant = Move::new(sq("e5"), sq("d6"));
    // This en passant is illegal because it exposes the king.
    assert!(
        !game.is_legal_move(en_passant),
        "en passant exposing king should be illegal"
    );
    // Play a random game from this position — should not crash or violate invariants.
    let mut oracle = GameOracle::new(
        Box::new(RandomStrategy::new(42)),
        Box::new(RandomStrategy::new(99)),
    )
    .with_max_moves(20);
    let record = oracle.play_game(game);
    assert_no_violations(&record);
}

#[test]
fn double_check_allows_only_king_moves() {
    // Position with double check: bishop on b5 and rook on e1 both check black king on e8.
    let game = GameState::from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1")
        .expect("test FEN should parse");
    let legal = game.legal_moves();
    // All legal moves must be king moves.
    assert!(!legal.is_empty(), "must have at least one legal move");
    assert!(
        legal.iter().all(|m| m.from() == sq("e8")),
        "double check: only king moves should be legal, got: {legal:?}"
    );
    // Play from this position — should not violate invariants.
    let mut oracle = GameOracle::new(
        Box::new(RandomStrategy::new(42)),
        Box::new(RandomStrategy::new(99)),
    )
    .with_max_moves(20);
    let record = oracle.play_game(game);
    assert_no_violations(&record);
}

#[test]
fn fifty_move_rule_position_detects_draw_availability() {
    let game = GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 150 1")
        .expect("test FEN should parse");
    let mut oracle = GameOracle::new(
        Box::new(RandomStrategy::new(0)),
        Box::new(RandomStrategy::new(0)),
    );
    let record = oracle.play_game(game);
    assert_no_violations(&record);
    // Halfmove clock at 150 triggers automatic 75-move draw immediately.
    assert_eq!(record.move_count, 0);
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Draw(_))) => {}
        other => panic!("expected automatic draw, got {other:?}"),
    }
}
```

- [ ] **Step 4: Run all scripted scenario tests**

Run: `cargo test -p chess_core --features test-support --test game_oracle`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/chess_core/tests/game_oracle.rs
git commit -m "Add Layer 1 scripted scenario tests for game oracle"
```

---

### Task 7: Layer 1 — Random Game Batch Tests

**Files:**
- Modify: `crates/chess_core/tests/game_oracle.rs`

- [ ] **Step 1: Add random game batch test**

Append to `game_oracle.rs`:

```rust
// --- Random Game Batches ---

fn game_count() -> usize {
    std::env::var("CHESS_ORACLE_GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
}

fn base_seed() -> u64 {
    std::env::var("CHESS_ORACLE_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12345)
}

fn report_enabled() -> bool {
    std::env::var("CHESS_ORACLE_REPORT").is_ok()
}

#[test]
fn random_game_batch_completes_without_violations() {
    let count = game_count();
    let seed = base_seed();
    let write_reports = report_enabled();
    let report_dir = std::path::PathBuf::from("../../target/test-reports/oracle");

    let mut total_violations = 0;
    let mut failed_seeds = Vec::new();

    for i in 0..count {
        let game_seed = seed + i as u64;
        let strategy_name;
        let mut oracle = if i % 2 == 0 {
            strategy_name = "random";
            GameOracle::new(
                Box::new(RandomStrategy::new(game_seed)),
                Box::new(RandomStrategy::new(game_seed.wrapping_add(1))),
            )
        } else {
            strategy_name = "weighted";
            GameOracle::new(
                Box::new(WeightedStrategy::new(game_seed)),
                Box::new(WeightedStrategy::new(game_seed.wrapping_add(1))),
            )
        };

        let record = oracle.play_game(GameState::starting_position());

        if write_reports {
            let report =
                chess_core::testing::GameReport::from_record(&record, game_seed, strategy_name);
            let _ = report.write_to_dir(&report_dir, i);
        }

        if !record.violations.is_empty() {
            total_violations += record.violations.len();
            failed_seeds.push(game_seed);
            if !write_reports {
                panic!(
                    "Violation in game {i} (seed {game_seed}, strategy {strategy_name}):\n\
                     Initial FEN: {}\n\
                     Move count: {}\n\
                     Final FEN: {}\n\
                     Violations: {:?}\n\
                     Replay with: CHESS_ORACLE_SEED={game_seed} CHESS_ORACLE_GAMES=1",
                    record.initial_fen,
                    record.move_count,
                    record.final_fen,
                    record.violations,
                );
            }
        }
    }

    if write_reports && total_violations > 0 {
        panic!(
            "{total_violations} violation(s) across {} game(s). Failed seeds: {failed_seeds:?}\n\
             Reports written to {report_dir:?}",
            failed_seeds.len(),
        );
    }
}
```

- [ ] **Step 2: Run with default count (20 games)**

Run: `cargo test -p chess_core --features test-support --test game_oracle -- random_game_batch`
Expected: PASS (20 games, no violations, ~1 second).

- [ ] **Step 3: Run with elevated count to stress-test**

Run: `CHESS_ORACLE_GAMES=200 cargo test -p chess_core --features test-support --test game_oracle -- random_game_batch`
Expected: PASS (200 games, no violations).

- [ ] **Step 4: Commit**

```bash
git add crates/chess_core/tests/game_oracle.rs
git commit -m "Add configurable random game batch tests with env-var scaling and JSON reporting"
```

---

### Task 8: Add ClaimDraw to AutomationMatchAction (Layer 2 Prerequisite)

**Files:**
- Modify: `crates/game_app/src/automation.rs:153-166`
- Modify: `crates/game_app/src/plugins/input.rs:146-175`

- [ ] **Step 1: Write a test for ClaimDraw via automation**

Add a new test file or append to an existing automation test. For simplicity, we'll verify in the Layer 2 test file (Task 9). For now, add the variant and verify compilation.

Add `ClaimDraw` variant to `AutomationMatchAction` in `crates/game_app/src/automation.rs`:

```rust
pub enum AutomationMatchAction {
    SelectSquare {
        square: Square,
    },
    SubmitMove {
        from: Square,
        to: Square,
        promotion: Option<PieceKind>,
    },
    ChoosePromotion {
        piece: PieceKind,
    },
    ClearInteraction,
    ClaimDraw,
}
```

- [ ] **Step 2: Add dispatch arm in apply_match_action**

In `crates/game_app/src/plugins/input.rs`, function `apply_match_action`, add a new match arm:

```rust
        AutomationMatchAction::ClaimDraw => {
            if match_session.claim_draw() {
                Ok(())
            } else {
                Err(AutomationError::CommandIgnored(
                    String::from("no draw is claimable in the current position"),
                ))
            }
        }
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p game_app`
Expected: Compiles. If `automation-transport` feature needs serde support, the existing `serde(tag = "type", rename_all = "snake_case")` on the enum will auto-derive for the new unit variant.

- [ ] **Step 4: Verify existing tests still pass**

Run: `cargo test -p game_app`
Expected: All existing tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/game_app/src/automation.rs crates/game_app/src/plugins/input.rs
git commit -m "Add ClaimDraw variant to AutomationMatchAction for draw claim automation"
```

---

### Task 9: Layer 2 — Full-Stack Playthrough Tests

**Files:**
- Modify: `crates/game_app/Cargo.toml`
- Create: `crates/game_app/tests/automation_game_playthrough.rs`

- [ ] **Step 1: Enable test-support feature for game_app's chess_core dependency**

In `crates/game_app/Cargo.toml`, update the chess_core dev-dependency:

```toml
[dev-dependencies]
tempfile = "3.15.0"
chess_core = { path = "../chess_core", features = ["test-support"] }
```

- [ ] **Step 2: Create the test file with Scholar's Mate playthrough**

Create `crates/game_app/tests/automation_game_playthrough.rs`:

```rust
use tempfile::tempdir;

use chess_core::{
    GameOutcome, GameStatus, Move, PieceKind, Side, Square, WinReason, DrawReason,
};
use chess_persistence::{
    GameSnapshot, SaveKind, SessionStore, SnapshotMetadata, SnapshotShellState,
};
use game_app::{
    AutomationCommand, AutomationHarness, AutomationMatchAction, AutomationNavigationAction,
    AutomationSaveAction, AutomationScreen,
};

fn sq(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn submit_move(
    harness: &mut AutomationHarness,
    from: &str,
    to: &str,
    promotion: Option<PieceKind>,
) {
    harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
            from: sq(from),
            to: sq(to),
            promotion,
        }))
        .expect("move should succeed");
}

fn start_new_match(harness: &mut AutomationHarness) {
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::StartNewMatch,
        ))
        .expect("start command should succeed");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("match loading should settle");
}

// --- Scenario 1: Complete game to checkmate ---

#[test]
fn complete_game_to_checkmate_via_scholars_mate() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Scholar's Mate: 1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7#
    submit_move(&mut harness, "e2", "e4", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "d1", "h5", None);
    submit_move(&mut harness, "b8", "c6", None);
    submit_move(&mut harness, "f1", "c4", None);
    submit_move(&mut harness, "g8", "f6", None);
    submit_move(&mut harness, "h5", "f7", None);

    let snapshot = harness.snapshot();
    assert_eq!(snapshot.screen, AutomationScreen::MatchResult);
    match snapshot.match_state.status {
        GameStatus::Finished(GameOutcome::Win {
            winner: Side::White,
            reason: WinReason::Checkmate,
        }) => {}
        other => panic!("expected White checkmate, got {other:?}"),
    }
}
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test -p game_app --test automation_game_playthrough -- complete_game_to_checkmate`
Expected: PASS

- [ ] **Step 4: Add mid-game save/load scenario**

Append to the test file:

```rust
// --- Scenario 2: Mid-game save/load ---

#[test]
fn mid_game_save_load_preserves_position() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Play 4 moves: 1. e4 e5 2. Nf3 Nc6
    submit_move(&mut harness, "e2", "e4", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "g1", "f3", None);
    submit_move(&mut harness, "b8", "c6", None);

    let fen_before_save = harness.snapshot().match_state.fen.clone();

    // Save the game.
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SaveManual {
            label: Some(String::from("mid-game test")),
        }))
        .expect("save should succeed");

    // Return to menu.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::PauseMatch,
        ))
        .expect("pause should succeed");
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::ReturnToMenu,
        ))
        .expect("return should succeed");

    // Navigate to load list, select, and load.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenLoadList,
        ))
        .expect("open load list should succeed");

    let saves = harness.snapshot().saves.manual_saves.clone();
    assert!(!saves.is_empty(), "should have at least one save");
    let slot_id = saves[0].slot_id.clone();

    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id,
        }))
        .expect("select slot should succeed");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load should succeed");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("loading should settle");

    let fen_after_load = harness.snapshot().match_state.fen.clone();
    assert_eq!(fen_before_save, fen_after_load, "FEN should match after load");
}
```

- [ ] **Step 5: Add promotion scenario**

Append:

```rust
// --- Scenario 3: Promotion through automation ---

#[test]
fn promotion_through_automation_changes_piece_type() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    // Use a FEN with a pawn about to promote (need to set up via a loaded game).
    // Simplest: start a new match and use SubmitMove with promotion.
    // But we need a position where promotion is possible.
    // Pre-populate with a save at a promotable position.
    let mut store = SessionStore::new(root.path().to_path_buf());
    let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
    let game_state = chess_core::GameState::from_fen(fen).expect("test FEN");
    let snapshot = GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from("promotion-test"),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Manual,
            session_id: String::from("promo-test"),
            recovery_key: None,
        },
        SnapshotShellState::default(),
    );
    store
        .save_manual(snapshot)
        .expect("save should succeed");

    // Load the save.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenLoadList,
        ))
        .expect("open load list");
    let saves = harness.snapshot().saves.manual_saves.clone();
    let slot_id = saves[0].slot_id.clone();
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id,
        }))
        .expect("select");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    // Promote pawn to queen.
    submit_move(&mut harness, "a7", "a8", Some(PieceKind::Queen));

    let snapshot = harness.snapshot();
    // Verify the FEN shows a queen on a8.
    assert!(
        snapshot.match_state.fen.starts_with("Q3k3"),
        "FEN should show queen on a8, got: {}",
        snapshot.match_state.fen
    );
}
```

- [ ] **Step 6: Add draw claim flow scenario**

Append:

```rust
// --- Scenario 4: Draw claim flow ---

#[test]
fn draw_claim_through_automation_reaches_match_result() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    // Pre-populate a save with a 50-move-rule claimable position.
    let mut store = SessionStore::new(root.path().to_path_buf());
    let fen = "4k3/8/8/8/8/8/8/4K3 w - - 100 1";
    let game_state = chess_core::GameState::from_fen(fen).expect("test FEN");
    let snapshot = GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from("draw-claim-test"),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Manual,
            session_id: String::from("draw-test"),
            recovery_key: None,
        },
        SnapshotShellState::default(),
    );
    store.save_manual(snapshot).expect("save");

    // Load the position.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenLoadList,
        ))
        .expect("open load list");
    let saves = harness.snapshot().saves.manual_saves.clone();
    let slot_id = saves[0].slot_id.clone();
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id,
        }))
        .expect("select");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    // Verify draw is claimable.
    let snap = harness.snapshot();
    assert!(
        snap.match_state.claimable_draw.fifty_move_rule,
        "50-move rule should be claimable"
    );

    // Claim the draw.
    harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::ClaimDraw))
        .expect("claim draw should succeed");

    // Step to let the result transition happen.
    let snap = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("step");

    assert_eq!(snap.screen, AutomationScreen::MatchResult);
}
```

- [ ] **Step 7: Add rematch and stalemate scenarios**

Append:

```rust
// --- Scenario 5: Rematch after completion ---

#[test]
fn rematch_after_checkmate_starts_fresh_game() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Fool's Mate for quick finish.
    submit_move(&mut harness, "f2", "f3", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "g2", "g4", None);
    submit_move(&mut harness, "d8", "h4", None);

    assert_eq!(harness.snapshot().screen, AutomationScreen::MatchResult);

    // Rematch.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::Rematch,
        ))
        .expect("rematch should succeed");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    let snap = harness.snapshot();
    assert_eq!(snap.screen, AutomationScreen::InMatch);
    assert!(
        snap.match_state.fen.contains("rnbqkbnr"),
        "should be starting position after rematch"
    );
}

// --- Scenario 6: Stalemate through full stack ---

#[test]
fn stalemate_detected_through_full_stack() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    // Load a stalemate position.
    let mut store = SessionStore::new(root.path().to_path_buf());
    let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
    let game_state = chess_core::GameState::from_fen(fen).expect("test FEN");
    let snapshot = GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from("stalemate-test"),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Manual,
            session_id: String::from("stalemate-test"),
            recovery_key: None,
        },
        SnapshotShellState::default(),
    );
    store.save_manual(snapshot).expect("save");

    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenLoadList,
        ))
        .expect("open load list");
    let saves = harness.snapshot().saves.manual_saves.clone();
    let slot_id = saves[0].slot_id.clone();
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id,
        }))
        .expect("select");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    let snap = harness.snapshot();
    match snap.match_state.status {
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {}
        other => panic!("expected stalemate, got {other:?}"),
    }
}
```

- [ ] **Step 8: Add recovery resume scenario**

Append:

```rust
// --- Scenario 7: Recovery resume ---

#[test]
fn recovery_resume_preserves_position_after_simulated_crash() {
    let root = tempdir().expect("temp dir");

    // Pre-populate a recovery snapshot with a position reflecting 3 moves played.
    // Matches the existing test pattern in automation_semantic_flow.rs.
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    let mut store = SessionStore::new(root.path().to_path_buf());
    let game_state = chess_core::GameState::from_fen(fen).expect("test FEN");
    let snapshot = GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from("recovery-test"),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Recovery,
            session_id: String::from("recovery"),
            recovery_key: Some(String::from("autosave")),
        },
        SnapshotShellState::default(),
    );
    store.save_recovery(snapshot).expect("save recovery");

    // Boot a new harness — it should detect the recovery save.
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    let snap = harness.snapshot();
    assert!(
        snap.menu.recovery_available,
        "recovery banner should be available"
    );

    // Resume the recovery.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::ResumeRecovery,
        ))
        .expect("resume should succeed");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    let snap = harness.snapshot();
    assert_eq!(snap.screen, AutomationScreen::InMatch);
    assert_eq!(
        snap.match_state.fen, fen,
        "FEN should match the pre-populated recovery position"
    );
}
```

- [ ] **Step 9: Run all Layer 2 tests**

Run: `cargo test -p game_app --test automation_game_playthrough`
Expected: All 7 scenarios PASS.

- [ ] **Step 10: Commit**

```bash
git add crates/game_app/Cargo.toml crates/game_app/tests/automation_game_playthrough.rs
git commit -m "Add Layer 2 full-stack game playthrough tests via AutomationHarness"
```

---

### Task 10: Final Verification

**Files:** None (verification only)

- [ ] **Step 1: Run entire workspace test suite**

Run: `cargo test --workspace`
Expected: All tests pass, including the new ones.

- [ ] **Step 2: Run with test-support feature explicitly**

Run: `cargo test -p chess_core --features test-support`
Expected: All chess_core tests pass (existing + new oracle tests).

- [ ] **Step 3: Run heavy validation if available**

Run: `bash tools/ci/heavy-validation.sh` (if it exists)
Expected: PASS

- [ ] **Step 4: Run elevated game count to stress-test**

Run: `CHESS_ORACLE_GAMES=500 cargo test -p chess_core --features test-support --test game_oracle -- random_game_batch --nocapture`
Expected: 500 games complete with no violations.

- [ ] **Step 5: Check formatting**

Run: `cargo fmt --check`
Expected: No formatting issues.

- [ ] **Step 6: Check clippy**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: No warnings.

Also verify chess_core with test-support specifically:
Run: `cargo clippy -p chess_core --features test-support --all-targets -- -D warnings`
Expected: No warnings.

- [ ] **Step 7: Commit any formatting fixes if needed**

Only if Steps 5-6 required changes.
