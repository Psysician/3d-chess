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

    #[must_use]
    pub const fn pawn_forward(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }

    #[must_use]
    pub const fn home_rank(self) -> u8 {
        match self {
            Self::White => 0,
            Self::Black => 7,
        }
    }

    #[must_use]
    pub const fn pawn_start_rank(self) -> u8 {
        match self {
            Self::White => 1,
            Self::Black => 6,
        }
    }

    #[must_use]
    pub const fn promotion_rank(self) -> u8 {
        match self {
            Self::White => 7,
            Self::Black => 0,
        }
    }

    #[must_use]
    pub const fn fen_token(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    #[must_use]
    pub const fn fen_letter(self) -> char {
        match self {
            Self::King => 'k',
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Pawn => 'p',
        }
    }

    #[must_use]
    pub const fn is_valid_promotion(self) -> bool {
        matches!(self, Self::Queen | Self::Rook | Self::Bishop | Self::Knight)
    }
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

    #[must_use]
    pub fn from_fen(char_repr: char) -> Option<Self> {
        let side = if char_repr.is_ascii_uppercase() {
            Side::White
        } else {
            Side::Black
        };

        let kind = match char_repr.to_ascii_lowercase() {
            'k' => PieceKind::King,
            'q' => PieceKind::Queen,
            'r' => PieceKind::Rook,
            'b' => PieceKind::Bishop,
            'n' => PieceKind::Knight,
            'p' => PieceKind::Pawn,
            _ => return None,
        };

        Some(Self { side, kind })
    }

    #[must_use]
    pub const fn fen_char(self) -> char {
        let lower = self.kind.fen_letter();
        match self.side {
            Side::White => match lower {
                'k' => 'K',
                'q' => 'Q',
                'r' => 'R',
                'b' => 'B',
                'n' => 'N',
                'p' => 'P',
                _ => lower,
            },
            Side::Black => lower,
        }
    }
}
