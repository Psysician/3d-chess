use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Piece, Square};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardState {
    pieces: BTreeMap<Square, Piece>,
}

impl BoardState {
    #[must_use]
    pub fn place_piece(mut self, square: Square, piece: Piece) -> Self {
        self.pieces.insert(square, piece);
        self
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(&square).copied()
    }

    #[must_use]
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Square, &Piece)> {
        self.pieces.iter()
    }
}
