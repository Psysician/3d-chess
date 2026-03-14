use serde::{Deserialize, Serialize};

use crate::{BoardState, Side};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub board: BoardState,
    pub side_to_move: Side,
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: BoardState::default(),
            side_to_move: Side::White,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}
