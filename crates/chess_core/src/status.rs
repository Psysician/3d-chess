use serde::{Deserialize, Serialize};

use crate::Side;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DrawAvailability {
    pub threefold_repetition: bool,
    pub fifty_move_rule: bool,
}

impl DrawAvailability {
    #[must_use]
    pub const fn is_claimable(self) -> bool {
        self.threefold_repetition || self.fifty_move_rule
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomaticDrawReason {
    FivefoldRepetition,
    SeventyFiveMoveRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawReason {
    Stalemate,
    Automatic(AutomaticDrawReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinReason {
    Checkmate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOutcome {
    Win { winner: Side, reason: WinReason },
    Draw(DrawReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Ongoing {
        in_check: bool,
        draw_available: DrawAvailability,
    },
    Finished(GameOutcome),
}

impl GameStatus {
    #[must_use]
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Finished(_))
    }
}
