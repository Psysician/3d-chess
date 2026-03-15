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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square;

    #[test]
    fn move_display_formats_basic_and_promotion_uci() {
        let e2 = Square::from_algebraic("e2").expect("valid square");
        let e4 = Square::from_algebraic("e4").expect("valid square");
        let a7 = Square::from_algebraic("a7").expect("valid square");
        let a8 = Square::from_algebraic("a8").expect("valid square");

        let basic = Move::new(e2, e4);
        assert_eq!(basic.from(), e2);
        assert_eq!(basic.to(), e4);
        assert_eq!(basic.promotion(), None);
        assert_eq!(basic.to_string(), "e2e4");

        let promotion = Move::with_promotion(a7, a8, PieceKind::Queen);
        assert_eq!(promotion.promotion(), Some(PieceKind::Queen));
        assert_eq!(promotion.to_string(), "a7a8q");
        assert_eq!(
            Move::with_promotion(a7, a8, PieceKind::Knight).to_string(),
            "a7a8n"
        );
    }

    #[test]
    fn move_error_messages_stay_stable() {
        assert_eq!(
            MoveError::GameAlreadyFinished.to_string(),
            "game is already finished"
        );
        assert_eq!(
            MoveError::NoPieceAtSource.to_string(),
            "no piece at source square"
        );
        assert_eq!(
            MoveError::WrongSideToMove.to_string(),
            "piece does not match side to move"
        );
        assert_eq!(
            MoveError::MissingPromotionChoice.to_string(),
            "promotion move requires a promotion piece"
        );
        assert_eq!(
            MoveError::InvalidPromotionChoice.to_string(),
            "promotion piece must be queen, rook, bishop, or knight"
        );
    }
}
