use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Piece, Side, Square};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardState {
    pieces: BTreeMap<Square, Piece>,
}

impl BoardState {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn place_piece(mut self, square: Square, piece: Piece) -> Self {
        self.pieces.insert(square, piece);
        self
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.pieces.insert(square, piece);
    }

    pub fn remove_piece(&mut self, square: Square) -> Option<Piece> {
        self.pieces.remove(&square)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(&square).copied()
    }

    #[must_use]
    pub fn contains_piece(&self, square: Square) -> bool {
        self.pieces.contains_key(&square)
    }

    #[must_use]
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Square, &Piece)> {
        self.pieces.iter()
    }

    pub fn iter_side(&self, side: Side) -> impl Iterator<Item = (Square, Piece)> + '_ {
        self.pieces
            .iter()
            .filter(move |(_, piece)| piece.side == side)
            .map(|(square, piece)| (*square, *piece))
    }

    #[must_use]
    pub fn king_square(&self, side: Side) -> Option<Square> {
        self.iter_side(side)
            .find(|(_, piece)| piece.kind == crate::PieceKind::King)
            .map(|(square, _)| square)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PieceKind, Side};

    #[test]
    fn board_state_places_removes_and_filters_pieces() {
        let white_king = Piece::new(Side::White, PieceKind::King);
        let black_king = Piece::new(Side::Black, PieceKind::King);
        let e1 = Square::from_coords_unchecked(4, 0);
        let e8 = Square::from_coords_unchecked(4, 7);

        let mut board = BoardState::empty().place_piece(e1, white_king);
        board.set_piece(e8, black_king);

        assert!(board.contains_piece(e1));
        assert_eq!(board.piece_count(), 2);
        assert_eq!(board.piece_at(e8), Some(black_king));
        assert_eq!(board.king_square(Side::White), Some(e1));
        assert_eq!(board.king_square(Side::Black), Some(e8));

        let white_pieces = board.iter_side(Side::White).collect::<Vec<_>>();
        assert_eq!(white_pieces, vec![(e1, white_king)]);
        assert_eq!(board.iter().count(), 2);
        assert_eq!(board.remove_piece(e1), Some(white_king));
        assert!(!board.contains_piece(e1));
    }
}
