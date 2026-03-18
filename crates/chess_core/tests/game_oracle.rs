use chess_core::testing::{
    BatchStats, GameOracle, GameRecord, GameReport, GameTermination, RandomStrategy,
    ScriptedStrategy, WeightedStrategy,
};
use chess_core::{
    DrawReason, GameOutcome, GameState, GameStatus, Move, PieceKind, Side, Square, WinReason,
};

fn square(name: &str) -> Square {
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
                Move::new(square("e2"), square("e4")),
                Move::new(square("d1"), square("h5")),
                Move::new(square("f1"), square("c4")),
                Move::new(square("h5"), square("f7")),
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(square("e7"), square("e5")),
                Move::new(square("b8"), square("c6")),
                Move::new(square("g8"), square("f6")),
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
                Move::new(square("f2"), square("f3")),
                Move::new(square("g2"), square("g4")),
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(square("e7"), square("e5")),
                Move::new(square("d8"), square("h4")),
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
                Move::new(square("e2"), square("e4")),
                Move::new(square("e4"), square("e5")),
                Move::new(square("e5"), square("f6")),
            ],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![
                Move::new(square("d7"), square("d5")),
                Move::new(square("f7"), square("f5")),
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
            vec![Move::new(square("e1"), square("g1"))],
            0,
        )),
        Box::new(ScriptedStrategy::new(
            vec![Move::new(square("e8"), square("c8"))],
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
        let game =
            GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("test FEN should parse");
        let mut oracle = GameOracle::new(
            Box::new(ScriptedStrategy::new(
                vec![Move::with_promotion(square("a7"), square("a8"), piece)],
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
fn promotion_with_capture_completes_without_violations() {
    // White pawn on a7, black rook on b8. Pawn captures rook and promotes to queen.
    let game =
        GameState::from_fen("1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("test FEN should parse");
    let mut oracle = GameOracle::new(
        Box::new(ScriptedStrategy::new(
            vec![Move::with_promotion(
                square("a7"),
                square("b8"),
                PieceKind::Queen,
            )],
            0,
        )),
        Box::new(RandomStrategy::new(42)),
    )
    .with_max_moves(2);
    let record = oracle.play_game(game);
    assert_no_violations(&record);
}

#[test]
fn stalemate_position_detects_draw() {
    let game =
        GameState::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").expect("test FEN should parse");
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
    let start =
        GameState::from_fen("4k3/8/8/8/8/8/N7/4K3 w - - 0 1").expect("test FEN should parse");
    let cycle_white = [
        Move::new(square("a2"), square("b4")),
        Move::new(square("b4"), square("a2")),
    ];
    let cycle_black = [
        Move::new(square("e8"), square("d8")),
        Move::new(square("d8"), square("e8")),
    ];
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
    let game =
        GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 150 1").expect("test FEN should parse");
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
    let game =
        GameState::from_fen("4r1k1/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("test FEN should parse");
    let en_passant = Move::new(square("e5"), square("d6"));
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
fn double_check_position_plays_without_violations() {
    let game =
        GameState::from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1").expect("test FEN should parse");
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
    let report_dir = std::env::var("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("../../target"))
        .join("test-reports/oracle");

    let mut stats = BatchStats::default();
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
        stats.record(&record);

        if write_reports {
            let report = GameReport::from_record(&record, game_seed, strategy_name);
            let _ = report.write_to_dir(&report_dir, i);
        }

        if !record.violations.is_empty() {
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

    eprintln!("Oracle batch: {stats}");

    if write_reports && stats.total_violations > 0 {
        panic!(
            "{} violation(s) across {} game(s). Failed seeds: {failed_seeds:?}\n\
             Reports written to {report_dir:?}",
            stats.total_violations,
            failed_seeds.len(),
        );
    }
}
