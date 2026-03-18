use chess_core::testing::{
    GameOracle, GameRecord, GameReport, GameTermination, RandomStrategy, ScriptedStrategy,
    WeightedStrategy,
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

// ---------------------------------------------------------------------------
// Scripted Scenarios
// ---------------------------------------------------------------------------

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

#[test]
fn en_passant_capture_completes_without_violations() {
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
    assert!(
        record.move_count >= 5,
        "should play at least the scripted moves"
    );
}

#[test]
fn castling_both_sides_completes_without_violations() {
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
    for piece in [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
    ] {
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
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Draw(_))) => {}
        other => panic!("expected draw termination, got {other:?}"),
    }
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
    assert_eq!(record.move_count, 0);
    match &record.termination {
        GameTermination::Completed(GameStatus::Finished(GameOutcome::Draw(_))) => {}
        other => panic!("expected automatic draw, got {other:?}"),
    }
}

#[test]
fn en_passant_exposing_king_is_not_played() {
    let game = GameState::from_fen("4r1k1/8/8/3pP3/8/8/8/4K3 w - d6 0 1")
        .expect("test FEN should parse");
    let en_passant = Move::new(sq("e5"), sq("d6"));
    assert!(
        !game.is_legal_move(en_passant),
        "en passant exposing king should be illegal"
    );
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
    let game = GameState::from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1")
        .expect("test FEN should parse");
    let legal = game.legal_moves();
    assert!(!legal.is_empty(), "must have at least one legal move");
    assert!(
        legal.iter().all(|m| m.from() == sq("e8")),
        "double check: only king moves should be legal, got: {legal:?}"
    );
    let mut oracle = GameOracle::new(
        Box::new(RandomStrategy::new(42)),
        Box::new(RandomStrategy::new(99)),
    )
    .with_max_moves(20);
    let record = oracle.play_game(game);
    assert_no_violations(&record);
}

// ---------------------------------------------------------------------------
// Random Game Batches
// ---------------------------------------------------------------------------

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
            let report = GameReport::from_record(&record, game_seed, strategy_name);
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
                    record.initial_fen, record.move_count, record.final_fen, record.violations,
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
