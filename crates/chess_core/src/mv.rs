use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{PieceKind, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<PieceKind>,
}

impl Move {
    #[must_use]
    pub const fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    #[must_use]
    pub const fn with_promotion(from: Square, to: Square, promotion: PieceKind) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }

    #[must_use]
    pub const fn from(self) -> Square {
        self.from
    }

    #[must_use]
    pub const fn to(self) -> Square {
        self.to
    }

    #[must_use]
    pub const fn promotion(self) -> Option<PieceKind> {
        self.promotion
    }
}

impl Display for Move {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.from.to_string())?;
        formatter.write_str(&self.to.to_string())?;

        if let Some(promotion) = self.promotion {
            formatter.write_str(match promotion {
                PieceKind::Queen => "q",
                PieceKind::Rook => "r",
                PieceKind::Bishop => "b",
                PieceKind::Knight => "n",
                PieceKind::King | PieceKind::Pawn => "",
            })?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    GameAlreadyFinished,
    NoPieceAtSource,
    WrongSideToMove,
    IllegalMove,
    MissingPromotionChoice,
    InvalidPromotionChoice,
}

impl Display for MoveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::GameAlreadyFinished => "game is already finished",
            Self::NoPieceAtSource => "no piece at source square",
            Self::WrongSideToMove => "piece does not match side to move",
            Self::IllegalMove => "move is not legal in this position",
            Self::MissingPromotionChoice => "promotion move requires a promotion piece",
            Self::InvalidPromotionChoice => {
                "promotion piece must be queen, rook, bishop, or knight"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for MoveError {}
