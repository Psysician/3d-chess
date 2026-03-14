use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    White,
    Black,
}

impl Side {
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
}

impl Piece {
    #[must_use]
    pub const fn new(side: Side, kind: PieceKind) -> Self {
        Self { side, kind }
    }
}
