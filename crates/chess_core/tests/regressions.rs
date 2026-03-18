use chess_core::{GameState, Move, MoveError, Piece, PieceKind, Side, Square};

fn square(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

#[test]
fn no_piece_at_source_reports_a_distinct_error() {
    let start = GameState::starting_position();

    assert_eq!(
        start.apply_move(Move::new(square("e3"), square("e4"))),
        Err(MoveError::NoPieceAtSource)
    );
}

#[test]
fn plain_promotion_move_is_rejected_by_public_move_application() {
    let game = GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("FEN should parse");

    assert_eq!(
        game.apply_move(Move::new(square("a7"), square("a8"))),
        Err(MoveError::IllegalMove)
    );
}

#[test]
fn invalid_promotion_piece_is_rejected_by_public_move_application() {
    let game = GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("FEN should parse");

    assert_eq!(
        game.apply_move(Move::with_promotion(
            square("a7"),
            square("a8"),
            PieceKind::King,
        )),
        Err(MoveError::IllegalMove)
    );
    assert_eq!(
        game.apply_move(Move::with_promotion(
            square("a7"),
            square("a8"),
            PieceKind::Pawn,
        )),
        Err(MoveError::IllegalMove)
    );
}

#[test]
fn queenside_castle_is_generated_and_updates_state() {
    let game = GameState::from_fen("r3k3/8/8/8/8/8/8/R3K3 w Qq - 0 1").expect("FEN should parse");

    let castle = Move::new(square("e1"), square("c1"));
    assert!(game.is_legal_move(castle));

    let next = game
        .apply_move(castle)
        .expect("queenside castling should be legal");

    assert_eq!(
        next.piece_at(square("c1")),
        Some(Piece::new(Side::White, PieceKind::King))
    );
    assert_eq!(
        next.piece_at(square("d1")),
        Some(Piece::new(Side::White, PieceKind::Rook))
    );
    assert_eq!(next.to_fen(), "r3k3/8/8/8/8/8/8/2KR4 b q - 1 1");
}
