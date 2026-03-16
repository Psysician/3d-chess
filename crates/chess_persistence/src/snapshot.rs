//! Versioned saved-session snapshots for manual saves and interrupted-session recovery.
//! Domain state stays authoritative while shell metadata captures legality-critical interaction state. (ref: DL-002) (ref: DL-004)

use chess_core::{GameState, Move, Square};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SaveFormatVersion {
    V1,
    #[default]
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SaveKind {
    #[default]
    Manual,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimedDrawSnapshot {
    ThreefoldRepetition,
    FiftyMoveRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingPromotionSnapshot {
    pub from: Square,
    pub to: Square,
}

/// Captures the minimal interaction state that must survive restore without serializing Bevy entities.
/// Pending promotion and recovery dirtiness live here because they affect legal resume behavior. (ref: DL-002) (ref: DL-004)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SnapshotShellState {
    pub selected_square: Option<Square>,
    pub pending_promotion: Option<PendingPromotionSnapshot>,
    pub last_move: Option<Move>,
    pub claimed_draw: Option<ClaimedDrawSnapshot>,
    pub dirty_recovery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SnapshotMetadata {
    pub label: String,
    pub created_at_utc: Option<String>,
    #[serde(default)]
    pub updated_at_utc: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub save_kind: SaveKind,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub recovery_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub version: SaveFormatVersion,
    pub game_state: GameState,
    pub metadata: SnapshotMetadata,
    #[serde(default)]
    pub shell_state: SnapshotShellState,
}

impl GameSnapshot {
    #[must_use]
    pub fn new(game_state: GameState, metadata: SnapshotMetadata) -> Self {
        Self::from_parts(game_state, metadata, SnapshotShellState::default())
    }

    /// Builds a snapshot from domain state plus shell metadata so UI projections can be rebuilt after load.
    /// Restore behavior never depends on serializing Bevy world state. (ref: DL-004)
    #[must_use]
    pub fn from_parts(
        game_state: GameState,
        metadata: SnapshotMetadata,
        shell_state: SnapshotShellState,
    ) -> Self {
        Self {
            version: SaveFormatVersion::V2,
            game_state,
            metadata,
            shell_state,
        }
    }

    #[must_use]
    pub fn restore_game_state(&self) -> GameState {
        self.game_state.clone()
    }

    #[must_use]
    pub fn metadata(&self) -> &SnapshotMetadata {
        &self.metadata
    }

    #[must_use]
    pub fn shell_state(&self) -> &SnapshotShellState {
        &self.shell_state
    }
}
