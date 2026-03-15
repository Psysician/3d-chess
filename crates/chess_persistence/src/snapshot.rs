use chess_core::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SaveFormatVersion {
    #[default]
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SnapshotMetadata {
    pub label: String,
    pub created_at_utc: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub version: SaveFormatVersion,
    pub game_state: GameState,
    pub metadata: SnapshotMetadata,
}

impl GameSnapshot {
    #[must_use]
    pub fn new(game_state: GameState, metadata: SnapshotMetadata) -> Self {
        Self {
            version: SaveFormatVersion::V1,
            game_state,
            metadata,
        }
    }

    #[must_use]
    pub fn restore_game_state(&self) -> GameState {
        self.game_state.clone()
    }
}
