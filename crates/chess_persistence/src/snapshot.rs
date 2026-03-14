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
    pub fn placeholder(game_state: GameState) -> Self {
        Self {
            version: SaveFormatVersion::V1,
            game_state,
            metadata: SnapshotMetadata {
                label: String::from("m0-placeholder"),
                created_at_utc: None,
                notes: Some(String::from("Versioned save boundary reserved during M0.")),
            },
        }
    }
}
