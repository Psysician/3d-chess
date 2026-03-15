use chess_core::{
    AutomaticDrawReason, DrawReason, GameOutcome, GameState, GameStatus, Move, Piece, PieceKind,
    Side, Square, WinReason,
};

fn square(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn apply_moves(game: &GameState, moves: &[(&str, &str)]) -> GameState {
    let mut state = game.clone();

    for &(from, to) in moves {
        state = state
            .apply_move(Move::new(square(from), square(to)))
            .expect("test move should be legal");
    }

    state
}

#[test]
fn starting_position_has_exact_fen_and_20_legal_moves() {
    let game = GameState::starting_position();

    assert_eq!(
        game.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    assert_eq!(game.legal_moves().len(), 20);
}

#[test]
fn fen_roundtrip_preserves_complex_position() {
    let fen = "r3k2r/ppp2ppp/2n5/3pp3/3PP3/2N5/PPP2PPP/R3K2R w KQkq d6 7 12";
    let game = GameState::from_fen(fen).expect("complex FEN should parse");

    assert_eq!(game.to_fen(), fen);
}

#[test]
fn pinned_piece_cannot_move_off_the_file() {
    let game = GameState::from_fen("4r1k1/8/8/8/8/8/4R3/4K3 w - - 0 1").expect("FEN should parse");

    assert!(!game.is_legal_move(Move::new(square("e2"), square("d2"))));
    assert!(game.is_legal_move(Move::new(square("e2"), square("e8"))));
}

#[test]
fn double_check_allows_only_king_moves() {
    let game = GameState::from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1").expect("FEN should parse");

    let legal_moves = game.legal_moves();

    assert!(!legal_moves.is_empty());
    assert!(
        legal_moves
            .iter()
            .all(|candidate| candidate.from() == square("e8"))
    );
}

#[test]
fn kingside_castle_is_generated_and_updates_state() {
    let game = GameState::from_fen("4k2r/8/8/8/8/8/8/4K2R w Kk - 0 1").expect("FEN should parse");

    let castle = Move::new(square("e1"), square("g1"));
    assert!(game.is_legal_move(castle));

    let next = game.apply_move(castle).expect("castling should be legal");

    assert_eq!(
        next.piece_at(square("g1")),
        Some(Piece::new(Side::White, PieceKind::King))
    );
    assert_eq!(
        next.piece_at(square("f1")),
        Some(Piece::new(Side::White, PieceKind::Rook))
    );
    assert_eq!(next.to_fen(), "4k2r/8/8/8/8/8/8/5RK1 b k - 1 1");
}

#[test]
fn castling_through_check_is_not_legal() {
    let game = GameState::from_fen("4k2r/8/8/8/8/5r2/8/4K2R w Kk - 0 1").expect("FEN should parse");

    assert!(!game.is_legal_move(Move::new(square("e1"), square("g1"))));
}

#[test]
fn en_passant_capture_is_generated_and_applied() {
    let game = GameState::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("FEN should parse");
    let en_passant = Move::new(square("e5"), square("d6"));

    assert!(game.is_legal_move(en_passant));

    let next = game
        .apply_move(en_passant)
        .expect("en passant should be legal");

    assert_eq!(
        next.piece_at(square("d6")),
        Some(Piece::new(Side::White, PieceKind::Pawn))
    );
    assert_eq!(next.piece_at(square("d5")), None);
    assert_eq!(next.to_fen(), "4k3/8/3P4/8/8/8/8/4K3 b - - 0 1");
}

#[test]
fn en_passant_that_exposes_the_king_is_illegal() {
    let game =
        GameState::from_fen("4r1k1/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("FEN should parse");

    assert!(!game.is_legal_move(Move::new(square("e5"), square("d6"))));
}

#[test]
fn promotion_requires_choice_and_applies_selected_piece() {
    let game = GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("FEN should parse");
    let plain = Move::new(square("a7"), square("a8"));
    let promote = Move::with_promotion(square("a7"), square("a8"), PieceKind::Queen);

    assert!(!game.is_legal_move(plain));
    assert!(game.is_legal_move(promote));

    let next = game.apply_move(promote).expect("promotion should be legal");

    assert_eq!(
        next.piece_at(square("a8")),
        Some(Piece::new(Side::White, PieceKind::Queen))
    );
    assert_eq!(next.to_fen(), "Q3k3/8/8/8/8/8/8/4K3 b - - 0 1");
}

#[test]
fn checkmate_is_reported_as_a_terminal_win() {
    let game = GameState::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("FEN should parse");

    assert_eq!(
        game.status(),
        GameStatus::Finished(GameOutcome::Win {
            winner: Side::White,
            reason: WinReason::Checkmate,
        })
    );
}

#[test]
fn stalemate_is_reported_as_a_terminal_draw() {
    let game = GameState::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").expect("FEN should parse");

    assert_eq!(
        game.status(),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate))
    );
}

#[test]
fn repetition_progresses_from_claimable_to_automatic() {
    let start = GameState::from_fen("4k3/8/8/8/8/8/N7/4K3 w - - 0 1").expect("FEN should parse");
    let cycle = [("a2", "b4"), ("e8", "d8"), ("b4", "a2"), ("d8", "e8")];

    let threefold = apply_moves(&apply_moves(&start, &cycle), &cycle);
    match threefold.status() {
        GameStatus::Ongoing { draw_available, .. } => {
            assert!(draw_available.threefold_repetition);
            assert!(!draw_available.fifty_move_rule);
        }
        other => panic!("expected ongoing game with claimable draw, got {other:?}"),
    }

    let fivefold = apply_moves(&apply_moves(&threefold, &cycle), &cycle);
    assert_eq!(
        fivefold.status(),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::FivefoldRepetition,
        )))
    );
}

#[test]
fn fifty_and_seventy_five_move_rules_are_distinct() {
    let claimable =
        GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1").expect("FEN should parse");
    let automatic =
        GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 150 1").expect("FEN should parse");

    match claimable.status() {
        GameStatus::Ongoing { draw_available, .. } => {
            assert!(draw_available.fifty_move_rule);
            assert!(!draw_available.threefold_repetition);
        }
        other => panic!("expected ongoing claimable-draw state, got {other:?}"),
    }

    assert_eq!(
        automatic.status(),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::SeventyFiveMoveRule,
        )))
    );
}
