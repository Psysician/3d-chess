//! Bevy-facing match bridge for local play, load, and recovery flows.
//! Snapshot conversion keeps `chess_core` authoritative while the shell restores only the interaction state it needs. (ref: DL-001) (ref: DL-004)

use bevy::prelude::Resource;
use chess_core::{DrawAvailability, GameState, GameStatus, Move, MoveError, Piece, Square};
use chess_persistence::{
    ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SnapshotMetadata,
    SnapshotShellState,
};

/// Describes how MatchLoading should hydrate the next match without exploding top-level screen routing. (ref: DL-001)
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchLaunchIntent {
    #[default]
    NewLocalMatch,
    LoadManual,
    ResumeRecovery,
    Rematch,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PendingLoadedSnapshot(pub Option<GameSnapshot>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSessionSummary {
    pub status: GameStatus,
    pub last_move: Option<Move>,
    pub pending_promotion: bool,
    pub dirty_recovery: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimedDrawReason {
    ThreefoldRepetition,
    FiftyMoveRule,
}

// MatchSession stays the only Bevy-facing bridge to chess_core.
// Snapshot conversion keeps save/load from promoting Bevy shell state into domain authority.
#[derive(Resource, Debug, Clone)]
pub struct MatchSession {
    pub game_state: GameState,
    pub selected_square: Option<Square>,
    pub pending_promotion_move: Option<Move>,
    pub last_move: Option<Move>,
    pub claimed_draw: Option<ClaimedDrawReason>,
    dirty_recovery: bool,
}

impl MatchSession {
    #[must_use]
    pub fn start_local_match() -> Self {
        Self {
            game_state: GameState::starting_position(),
            selected_square: None,
            pending_promotion_move: None,
            last_move: None,
            claimed_draw: None,
            dirty_recovery: true,
        }
    }

    pub fn reset_for_local_match(&mut self) {
        *self = Self::start_local_match();
    }

    /// Restores a playable session from persisted domain and shell metadata.
    /// Bevy interaction state rebuilds from the snapshot instead of acting as a second source of truth. (ref: DL-004)
    #[must_use]
    pub fn restore_from_snapshot(snapshot: &GameSnapshot) -> Self {
        let claimed_draw = snapshot
            .shell_state()
            .claimed_draw
            .map(|reason| match reason {
                ClaimedDrawSnapshot::ThreefoldRepetition => ClaimedDrawReason::ThreefoldRepetition,
                ClaimedDrawSnapshot::FiftyMoveRule => ClaimedDrawReason::FiftyMoveRule,
            });

        Self {
            game_state: snapshot.restore_game_state(),
            selected_square: snapshot.shell_state().selected_square,
            pending_promotion_move: snapshot
                .shell_state()
                .pending_promotion
                .map(|promotion| Move::new(promotion.from, promotion.to)),
            last_move: snapshot.shell_state().last_move,
            claimed_draw,
            dirty_recovery: snapshot.shell_state().dirty_recovery,
        }
    }

    /// Produces the persisted session contract that save/load plugins hand to the repository boundary. (ref: DL-002) (ref: DL-004)
    #[must_use]
    pub fn to_snapshot(&self, metadata: SnapshotMetadata) -> GameSnapshot {
        let claimed_draw = self.claimed_draw.map(|reason| match reason {
            ClaimedDrawReason::ThreefoldRepetition => ClaimedDrawSnapshot::ThreefoldRepetition,
            ClaimedDrawReason::FiftyMoveRule => ClaimedDrawSnapshot::FiftyMoveRule,
        });

        GameSnapshot::from_parts(
            self.game_state.clone(),
            metadata,
            SnapshotShellState {
                selected_square: self.selected_square,
                pending_promotion: self.pending_promotion_move.map(|promotion| {
                    PendingPromotionSnapshot {
                        from: promotion.from(),
                        to: promotion.to(),
                    }
                }),
                last_move: self.last_move,
                claimed_draw,
                dirty_recovery: self.dirty_recovery,
            },
        )
    }

    /// Summarizes only shell-relevant facts so UI can render status without reaching through gameplay internals. (ref: DL-007)
    #[must_use]
    pub fn summary(&self) -> MatchSessionSummary {
        MatchSessionSummary {
            status: self.status(),
            last_move: self.last_move,
            pending_promotion: self.pending_promotion_move.is_some(),
            dirty_recovery: self.dirty_recovery,
        }
    }

    #[must_use]
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn replace_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
        self.last_move = None;
        self.claimed_draw = None;
        self.clear_interaction();
        self.mark_recovery_dirty();
    }

    #[must_use]
    pub fn legal_targets_for_selected(&self) -> Vec<Square> {
        let Some(selected_square) = self.selected_square else {
            return Vec::new();
        };

        self.game_state
            .legal_moves()
            .into_iter()
            .filter(|candidate| candidate.from() == selected_square)
            .map(Move::to)
            .collect()
    }

    #[must_use]
    pub fn status(&self) -> GameStatus {
        self.game_state.status()
    }

    #[must_use]
    pub fn claimable_draw(&self) -> DrawAvailability {
        self.game_state.draw_availability()
    }

    pub fn clear_interaction(&mut self) {
        self.selected_square = None;
        self.pending_promotion_move = None;
    }

    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.game_state.piece_at(square)
    }

    pub fn apply_move(&mut self, candidate: Move) -> Result<(), MoveError> {
        let next_state = self.game_state.apply_move(candidate)?;
        self.game_state = next_state;
        self.last_move = Some(candidate);
        self.claimed_draw = None;
        self.clear_interaction();
        self.mark_recovery_dirty();
        Ok(())
    }

    #[must_use]
    pub fn claimed_draw_reason(&self) -> Option<ClaimedDrawReason> {
        self.claimed_draw
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.claimed_draw.is_some() || self.status().is_finished()
    }

    #[must_use]
    pub fn is_recovery_dirty(&self) -> bool {
        self.dirty_recovery
    }

    pub fn mark_recovery_dirty(&mut self) {
        self.dirty_recovery = true;
    }

    pub fn mark_recovery_persisted(&mut self) {
        self.dirty_recovery = false;
    }

    pub fn claim_draw(&mut self) -> bool {
        let availability = self.claimable_draw();
        let reason = if availability.threefold_repetition {
            Some(ClaimedDrawReason::ThreefoldRepetition)
        } else if availability.fifty_move_rule {
            Some(ClaimedDrawReason::FiftyMoveRule)
        } else {
            None
        };

        let Some(reason) = reason else {
            return false;
        };

        self.claimed_draw = Some(reason);
        self.clear_interaction();
        self.mark_recovery_dirty();
        true
    }
}

impl Default for MatchSession {
    fn default() -> Self {
        Self::start_local_match()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_persistence::{GameSnapshot, PendingPromotionSnapshot, SaveKind};

    fn square(name: &str) -> Square {
        Square::from_algebraic(name).expect("test square must be valid")
    }

    fn sample_snapshot(dirty_recovery: bool) -> GameSnapshot {
        GameSnapshot::from_parts(
            GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1")
                .expect("fixture FEN should parse"),
            SnapshotMetadata {
                label: String::from("Fixture"),
                created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
                updated_at_utc: None,
                notes: None,
                save_kind: SaveKind::Manual,
                session_id: String::from("fixture"),
                recovery_key: None,
            },
            SnapshotShellState {
                selected_square: Some(square("e7")),
                pending_promotion: Some(PendingPromotionSnapshot {
                    from: square("e7"),
                    to: square("e8"),
                }),
                last_move: Some(Move::new(square("e7"), square("e8"))),
                claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
                dirty_recovery,
            },
        )
    }

    #[test]
    fn restore_from_snapshot_keeps_shell_bridge_fields() {
        let session = MatchSession::restore_from_snapshot(&sample_snapshot(true));
        assert_eq!(session.selected_square, Some(square("e7")));
        assert_eq!(
            session.pending_promotion_move,
            Some(Move::new(square("e7"), square("e8")))
        );
        assert_eq!(
            session.claimed_draw_reason(),
            Some(ClaimedDrawReason::ThreefoldRepetition)
        );
        assert!(session.summary().pending_promotion);
        assert!(session.summary().dirty_recovery);
    }

    #[test]
    fn replace_game_state_clears_interaction_and_marks_recovery_dirty() {
        let mut session = MatchSession::restore_from_snapshot(&sample_snapshot(false));
        session.replace_game_state(GameState::starting_position());

        assert_eq!(session.selected_square, None);
        assert_eq!(session.pending_promotion_move, None);
        assert_eq!(session.last_move, None);
        assert!(session.is_recovery_dirty());
    }

    #[test]
    fn claim_draw_updates_summary_without_reaching_through_game_state() {
        let mut session = MatchSession::start_local_match();
        session.replace_game_state(
            GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1")
                .expect("fixture FEN should parse"),
        );
        assert!(session.claim_draw());
        assert!(session.is_finished());
        assert!(session.summary().dirty_recovery);
    }
}
