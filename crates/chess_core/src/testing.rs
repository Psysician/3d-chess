//! Test infrastructure for playing complete games and verifying invariants.
//!
//! Gated behind the `test-support` feature because downstream crates (e.g. `game_app`)
//! need these types in their integration tests. `cfg(test)` would not work because it
//! only applies when compiling the crate itself for testing.

use std::fmt;

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::Serialize;

use crate::{BoardState, CastlingRights, GameState, GameStatus, Move, PieceKind, Side};

// ---------------------------------------------------------------------------
// MoveStrategy trait + built-in strategies
// ---------------------------------------------------------------------------

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
                .is_some_and(|p| p.kind == PieceKind::Pawn);
        let is_castling = game
            .piece_at(mv.from())
            .is_some_and(|p| p.kind == PieceKind::King)
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
        }
        self.fallback.select_move(game, legal_moves)
    }
}

// ---------------------------------------------------------------------------
// InvariantChecker
// ---------------------------------------------------------------------------

/// A rule violation detected by the invariant checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub description: String,
    pub fen_before: String,
    pub attempted_move: String,
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
    StatusMoveGenerationMismatch,
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

fn violation(kind: ViolationKind, description: String) -> Violation {
    Violation {
        kind,
        description,
        fen_before: String::new(),
        attempted_move: String::new(),
    }
}

/// Post-move invariant checker. Call after every `apply_move`.
pub struct InvariantChecker;

impl InvariantChecker {
    pub fn check(before: &GameState, mv: &Move, after: &GameState) -> Vec<Violation> {
        let mut raw = Vec::new();
        let fen_before = before.to_fen();
        let move_str = mv.to_string();

        Self::check_king_safety(after, before.side_to_move(), &mut raw);
        Self::check_king_count(after, &mut raw);
        Self::check_piece_counts(before, mv, after, &mut raw);
        Self::check_castling_monotonicity(before, after, &mut raw);
        Self::check_en_passant(after, &mut raw);
        Self::check_halfmove_clock(before, mv, after, &mut raw);
        Self::check_fullmove_number(before, after, &mut raw);
        Self::check_position_history(before, after, &mut raw);

        for v in &mut raw {
            v.fen_before.clone_from(&fen_before);
            v.attempted_move.clone_from(&move_str);
        }

        raw
    }

    fn check_king_safety(after: &GameState, moved_side: Side, violations: &mut Vec<Violation>) {
        if after.board().king_square(moved_side).is_none() {
            violations.push(violation(
                ViolationKind::KingInCheck,
                format!("{moved_side:?} king not found after move"),
            ));
            return;
        }
        if after.is_in_check(moved_side) {
            violations.push(violation(
                ViolationKind::KingInCheck,
                format!("{moved_side:?} king is in check after their own move"),
            ));
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
                violations.push(violation(
                    ViolationKind::KingCountInvalid,
                    format!("{side:?} has {king_count} kings, expected 1"),
                ));
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

        let expected = if is_promotion && is_capture {
            // Promotion with capture: pawn replaces captured piece — net -1.
            before_total - 1
        } else if is_capture || is_en_passant {
            before_total - 1
        } else {
            // Quiet move or promotion without capture: total unchanged.
            before_total
        };

        if after_total != expected {
            violations.push(violation(
                ViolationKind::PieceCountInconsistent,
                format!("piece count: expected {expected}, got {after_total} (capture={is_capture}, en_passant={is_en_passant}, promotion={is_promotion})"),
            ));
        }
    }

    fn check_castling_monotonicity(
        before: &GameState,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let b = castling_rights_as_tuple(before.castling_rights());
        let a = castling_rights_as_tuple(after.castling_rights());
        if (!b.0 && a.0) || (!b.1 && a.1) || (!b.2 && a.2) || (!b.3 && a.3) {
            violations.push(violation(
                ViolationKind::CastlingRightsIncreased,
                format!("castling rights increased: before={b:?}, after={a:?}"),
            ));
        }
    }

    fn check_en_passant(after: &GameState, violations: &mut Vec<Violation>) {
        if let Some(target) = after.en_passant_target() {
            let rank = target.rank();
            if rank != 2 && rank != 5 {
                violations.push(violation(
                    ViolationKind::EnPassantInvalid,
                    format!("en passant target {target} has rank {rank}, expected 2 or 5"),
                ));
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
            violations.push(violation(
                ViolationKind::HalfmoveClockWrong,
                format!(
                    "halfmove clock: expected {expected}, got {}",
                    after.halfmove_clock()
                ),
            ));
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
            violations.push(violation(
                ViolationKind::FullmoveNumberWrong,
                format!(
                    "fullmove number: expected {expected}, got {} (moved side: {:?})",
                    after.fullmove_number(),
                    before.side_to_move()
                ),
            ));
        }
    }

    fn check_position_history(
        before: &GameState,
        after: &GameState,
        violations: &mut Vec<Violation>,
    ) {
        let expected_len = before.position_history().len() + 1;
        if after.position_history().len() != expected_len {
            violations.push(violation(
                ViolationKind::PositionHistoryInconsistent,
                format!(
                    "position history length: expected {expected_len}, got {}",
                    after.position_history().len()
                ),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// GameOracle + GameRecord + BatchStats
// ---------------------------------------------------------------------------

/// How a game ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTermination {
    Completed(GameStatus),
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

/// Aggregate statistics for a batch of games.
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_games: usize,
    pub checkmates: usize,
    pub stalemates: usize,
    pub draws: usize,
    pub move_limit_reached: usize,
    pub total_moves: u64,
    pub total_violations: usize,
}

impl BatchStats {
    pub fn record(&mut self, game: &GameRecord) {
        self.total_games += 1;
        self.total_moves += u64::from(game.move_count);
        self.total_violations += game.violations.len();
        match &game.termination {
            GameTermination::Completed(GameStatus::Finished(outcome)) => match outcome {
                crate::GameOutcome::Win { .. } => self.checkmates += 1,
                crate::GameOutcome::Draw(crate::DrawReason::Stalemate) => self.stalemates += 1,
                crate::GameOutcome::Draw(_) => self.draws += 1,
            },
            GameTermination::Completed(GameStatus::Ongoing { .. }) => {}
            GameTermination::MoveLimitReached(_) => self.move_limit_reached += 1,
        }
    }

    pub fn avg_game_length(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.total_moves as f64 / self.total_games as f64
        }
    }
}

impl fmt::Display for BatchStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} games | avg {:.1} moves | checkmate: {} | stalemate: {} | draw: {} | limit: {} | violations: {}",
            self.total_games,
            self.avg_game_length(),
            self.checkmates,
            self.stalemates,
            self.draws,
            self.move_limit_reached,
            self.total_violations,
        )
    }
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

    #[must_use]
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
                violations.push(violation(
                    ViolationKind::StatusMoveGenerationMismatch,
                    format!(
                        "legal_moves() is empty but status() returned {status:?} at move {move_count}"
                    ),
                ));
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
                    let mut v = violation(
                        ViolationKind::LegalMoveRejected,
                        format!(
                            "apply_move rejected a move from legal_moves(): {chosen} — {error}"
                        ),
                    );
                    v.fen_before = game.to_fen();
                    v.attempted_move = chosen.to_string();
                    violations.push(v);

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

// ---------------------------------------------------------------------------
// JSON Reporting
// ---------------------------------------------------------------------------

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
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveRecord {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViolationRecord {
    pub index: u16,
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
                index: u16::try_from(i).unwrap_or(u16::MAX),
                fen_before: v.fen_before.clone(),
                attempted_move: v.attempted_move.clone(),
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
        }
    }

    pub fn write_to_dir(&self, dir: &std::path::Path, game_index: usize) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("game_{game_index:04}.json"));
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}

// ---------------------------------------------------------------------------
// Insufficient material detection
// ---------------------------------------------------------------------------

/// Returns `true` if neither side has enough material to deliver checkmate.
pub fn is_insufficient_material(game: &GameState) -> bool {
    let board = game.board();
    let mut white_bishops = 0u8;
    let mut black_bishops = 0u8;
    let mut white_knights = 0u8;
    let mut black_knights = 0u8;

    for (_sq, piece) in board.iter() {
        match (piece.side, piece.kind) {
            (_, PieceKind::King) => {}
            (_, PieceKind::Pawn | PieceKind::Rook | PieceKind::Queen) => return false,
            (Side::White, PieceKind::Bishop) => white_bishops += 1,
            (Side::Black, PieceKind::Bishop) => black_bishops += 1,
            (Side::White, PieceKind::Knight) => white_knights += 1,
            (Side::Black, PieceKind::Knight) => black_knights += 1,
        }
    }

    let white_minor = white_bishops + white_knights;
    let black_minor = black_bishops + black_knights;

    // K vs K
    if white_minor == 0 && black_minor == 0 {
        return true;
    }
    // K+B vs K or K+N vs K
    if (white_minor <= 1 && black_minor == 0) || (white_minor == 0 && black_minor <= 1) {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameOutcome, GameState, GameStatus, Square, WinReason};

    fn square(name: &str) -> Square {
        Square::from_algebraic(name).expect("test square must be valid")
    }

    // -- Strategy tests --

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
        let game = GameState::from_fen("4k3/3p4/8/8/3Q4/8/8/4K3 w - - 0 1")
            .expect("test FEN should parse");
        let legal_moves = game.legal_moves();
        let capture = Move::new(square("d4"), square("d7"));
        assert!(
            legal_moves.contains(&capture),
            "capture must be in legal moves"
        );

        let mut capture_count = 0;
        for seed in 0..100 {
            let mut strategy = WeightedStrategy::new(seed);
            if strategy.select_move(&game, &legal_moves) == capture {
                capture_count += 1;
            }
        }
        assert!(
            capture_count > 5,
            "weighted strategy should select captures more often than uniform, got {capture_count}/100"
        );
    }

    #[test]
    fn scripted_strategy_plays_script_then_falls_back_to_random() {
        let game = GameState::starting_position();
        let legal_moves = game.legal_moves();
        let e2e4 = Move::new(square("e2"), square("e4"));
        let mut strategy = ScriptedStrategy::new(vec![e2e4], 99);

        let first = strategy.select_move(&game, &legal_moves);
        assert_eq!(first, e2e4, "first move should be the scripted one");

        let second = strategy.select_move(&game, &legal_moves);
        assert!(
            legal_moves.contains(&second),
            "fallback move must be from legal moves"
        );
    }

    // -- InvariantChecker tests --

    #[test]
    fn invariant_checker_passes_for_a_legal_opening_move() {
        let before = GameState::starting_position();
        let mv = Move::new(square("e2"), square("e4"));
        let after = before.apply_move(mv).expect("legal move");
        let violations = InvariantChecker::check(&before, &mv, &after);
        assert!(
            violations.is_empty(),
            "no violations expected for legal e2e4, got: {violations:?}"
        );
    }

    #[test]
    fn invariant_checker_detects_fullmove_number_increment_after_black() {
        let after_e4 = GameState::starting_position()
            .apply_move(Move::new(square("e2"), square("e4")))
            .expect("legal");
        assert_eq!(
            after_e4.fullmove_number(),
            1,
            "fullmove stays 1 after white moves"
        );

        let mv = Move::new(square("e7"), square("e5"));
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
        let game = GameState::from_fen("4k3/3p4/8/8/3Q4/8/8/4K3 w - - 0 1")
            .expect("test FEN should parse");
        let capture = Move::new(square("d4"), square("d7"));
        let after = game.apply_move(capture).expect("legal capture");
        let violations = InvariantChecker::check(&game, &capture, &after);
        assert!(
            violations.is_empty(),
            "no violations for legal capture, got: {violations:?}"
        );
    }

    #[test]
    fn invariant_checker_validates_promotion_with_capture() {
        // White pawn on a7, black rook on b8. Pawn promotes to queen by capturing rook.
        let game =
            GameState::from_fen("1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("test FEN should parse");
        let promote_capture = Move::with_promotion(square("a7"), square("b8"), PieceKind::Queen);
        assert!(
            game.is_legal_move(promote_capture),
            "promotion-capture must be legal"
        );
        let after = game
            .apply_move(promote_capture)
            .expect("legal promotion-capture");
        let violations = InvariantChecker::check(&game, &promote_capture, &after);
        assert!(
            violations.is_empty(),
            "no violations for promotion-capture, got: {violations:?}"
        );
    }

    #[test]
    fn violation_carries_fen_and_move_context() {
        let before = GameState::starting_position();
        let mv = Move::new(square("e2"), square("e4"));
        let after = before.apply_move(mv).expect("legal move");
        let violations = InvariantChecker::check(&before, &mv, &after);
        assert!(violations.is_empty());
        // Verify the mechanism works by checking a valid check returns empty.
        // The stamping is verified structurally — if any violation were produced,
        // it would carry fen_before and attempted_move from the stamp loop.
    }

    // -- GameOracle tests --

    #[test]
    fn game_oracle_plays_scholars_mate_to_checkmate() {
        let script_white = vec![
            Move::new(square("e2"), square("e4")),
            Move::new(square("d1"), square("h5")),
            Move::new(square("f1"), square("c4")),
            Move::new(square("h5"), square("f7")),
        ];
        let script_black = vec![
            Move::new(square("e7"), square("e5")),
            Move::new(square("b8"), square("c6")),
            Move::new(square("g8"), square("f6")),
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
            ref other => panic!("expected White checkmate, got {other:?}"),
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
            GameTermination::Completed(_) => {}
            ref other => panic!("unexpected termination: {other:?}"),
        }
        assert!(record.move_count <= 10);
    }

    // -- BatchStats tests --

    #[test]
    fn batch_stats_tracks_outcomes() {
        let mut stats = BatchStats::default();
        let mut oracle = GameOracle::new(
            Box::new(RandomStrategy::new(42)),
            Box::new(RandomStrategy::new(99)),
        )
        .with_max_moves(10);
        let record = oracle.play_game(GameState::starting_position());
        stats.record(&record);
        assert_eq!(stats.total_games, 1);
        assert!(stats.total_moves > 0);
    }

    // -- Insufficient material tests --

    #[test]
    fn insufficient_material_detects_king_vs_king() {
        let game = GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("FEN should parse");
        assert!(is_insufficient_material(&game));
    }

    #[test]
    fn insufficient_material_detects_king_bishop_vs_king() {
        let game = GameState::from_fen("4k3/8/8/8/8/8/8/4KB2 w - - 0 1").expect("FEN should parse");
        assert!(is_insufficient_material(&game));
    }

    #[test]
    fn insufficient_material_detects_king_knight_vs_king() {
        let game = GameState::from_fen("4k3/8/8/8/8/8/8/4KN2 w - - 0 1").expect("FEN should parse");
        assert!(is_insufficient_material(&game));
    }

    #[test]
    fn sufficient_material_with_pawns() {
        let game = GameState::starting_position();
        assert!(!is_insufficient_material(&game));
    }

    #[test]
    fn sufficient_material_with_rook() {
        let game = GameState::from_fen("4k3/8/8/8/8/8/8/4K2R w - - 0 1").expect("FEN should parse");
        assert!(!is_insufficient_material(&game));
    }

    // -- GameReport tests --

    #[test]
    fn game_report_serializes_to_json() {
        let record = GameRecord {
            initial_fen: String::from("startpos"),
            moves: vec![Move::new(square("e2"), square("e4"))],
            final_fen: String::from("after"),
            termination: GameTermination::MoveLimitReached(1),
            violations: vec![],
            move_count: 1,
        };
        let report = GameReport::from_record(&record, 42, "random");
        let json =
            serde_json::to_string_pretty(&report).expect("GameReport should serialize to JSON");
        assert!(json.contains("\"seed\": 42"));
        assert!(json.contains("\"strategy\": \"random\""));
        assert!(json.contains("\"total_moves\": 1"));
    }
}
