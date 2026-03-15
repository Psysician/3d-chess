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
