//! Test infrastructure for playing complete games and verifying invariants.
//!
//! Gated behind the `test-support` feature because downstream crates (e.g. `game_app`)
//! need these types in their integration tests. `cfg(test)` would not work because it
//! only applies when compiling the crate itself for testing.

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::Serialize;

use crate::{BoardState, CastlingRights, GameState, GameStatus, Move, PieceKind, Side};

// --- MoveStrategy ---

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
        let weights: Vec<u32> = legal_moves
            .iter()
            .map(|mv| Self::weight_for(game, mv))
            .collect();
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

// --- InvariantChecker ---

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

fn total_piece_count(board: &BoardState) -> u8 {
    board
        .iter()
        .count()
        .try_into()
        .expect("piece count fits in u8")
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
        let before_total = total_piece_count(before.board());
        let after_total = total_piece_count(after.board());

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
                description: format!("castling rights increased: before={b:?}, after={a:?}"),
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
            || (moved_piece.kind == PieceKind::Pawn && before.en_passant_target() == Some(mv.to()));
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

// --- GameOracle ---

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

// --- GameReport ---

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
            .map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameOutcome, GameState, GameStatus, Square, WinReason};

    fn sq(name: &str) -> Square {
        Square::from_algebraic(name).expect("test square must be valid")
    }

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
        assert!(
            legal_moves.contains(&capture),
            "capture must be in legal moves"
        );

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

    #[test]
    fn invariant_checker_passes_for_a_legal_opening_move() {
        let before = GameState::starting_position();
        let mv = Move::new(
            Square::from_algebraic("e2").expect("valid"),
            Square::from_algebraic("e4").expect("valid"),
        );
        let after = before.apply_move(mv).expect("legal move");
        let violations = InvariantChecker::check(&before, &mv, &after);
        assert!(
            violations.is_empty(),
            "no violations expected for legal e2e4, got: {violations:?}"
        );
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
        assert_eq!(
            after_e4.fullmove_number(),
            1,
            "fullmove stays 1 after white moves"
        );

        let mv = Move::new(
            Square::from_algebraic("e7").expect("valid"),
            Square::from_algebraic("e5").expect("valid"),
        );
        let after_e5 = after_e4.apply_move(mv).expect("legal");
        assert_eq!(
            after_e5.fullmove_number(),
            2,
            "fullmove becomes 2 after black moves"
        );

        let violations = InvariantChecker::check(&after_e4, &mv, &after_e5);
        assert!(
            violations.is_empty(),
            "no violations for legal e7e5, got: {violations:?}"
        );
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
        assert!(
            violations.is_empty(),
            "no violations for legal capture, got: {violations:?}"
        );
    }

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

        assert!(
            record.violations.is_empty(),
            "no violations: {:?}",
            record.violations
        );
        assert_eq!(record.move_count, 7);
        match record.termination {
            GameTermination::Completed(GameStatus::Finished(GameOutcome::Win {
                winner: Side::White,
                reason: WinReason::Checkmate,
            })) => {}
            other => panic!("expected White checkmate, got {other:?}"),
        }
    }

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
}
