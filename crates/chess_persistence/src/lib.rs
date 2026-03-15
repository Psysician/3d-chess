pub mod snapshot;
pub mod store;

pub use snapshot::{
    ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SaveFormatVersion, SaveKind,
    SnapshotMetadata, SnapshotShellState,
};
pub use store::{
    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SavedSessionSummary, SessionStore,
    ShellSettings, StoreError, StoreResult,
};

#[cfg(test)]
mod tests {
    use chess_core::{GameState, GameStatus, Move, Square};

    use crate::{
        ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SaveFormatVersion, SaveKind,
        SnapshotMetadata, SnapshotShellState,
    };

    #[test]
    fn snapshot_roundtrips_and_preserves_legal_behavior() {
        let start = GameState::starting_position();
        let e2 = Square::from_algebraic("e2").expect("e2 must be valid");
        let e4 = Square::from_algebraic("e4").expect("e4 must be valid");
        let c7 = Square::from_algebraic("c7").expect("c7 must be valid");
        let c5 = Square::from_algebraic("c5").expect("c5 must be valid");

        let after_e4 = start
            .apply_move(Move::new(e2, e4))
            .expect("opening pawn move should be legal");
        let after_c5 = after_e4
            .apply_move(Move::new(c7, c5))
            .expect("reply pawn move should be legal");

        let shell_state = SnapshotShellState {
            selected_square: Some(c5),
            pending_promotion: Some(PendingPromotionSnapshot { from: e2, to: e4 }),
            last_move: Some(Move::new(c7, c5)),
            claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
            dirty_recovery: true,
        };
        let metadata = SnapshotMetadata {
            label: String::from("opening"),
            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
            updated_at_utc: Some(String::from("2026-03-15T00:05:00Z")),
            notes: Some(String::from("Persist opening state")),
            save_kind: SaveKind::Manual,
            session_id: String::from("opening"),
            recovery_key: None,
        };

        let snapshot = GameSnapshot::from_parts(after_c5.clone(), metadata.clone(), shell_state);
        let encoded =
            serde_json::to_string(&snapshot).expect("serializing the snapshot should succeed");
        let decoded: GameSnapshot =
            serde_json::from_str(&encoded).expect("deserializing the snapshot should succeed");
        let restored = decoded.restore_game_state();

        assert_eq!(
            decoded.shell_state(),
            &SnapshotShellState {
                selected_square: Some(c5),
                pending_promotion: Some(PendingPromotionSnapshot { from: e2, to: e4 }),
                last_move: Some(Move::new(c7, c5)),
                claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
                dirty_recovery: true,
            }
        );
        assert_eq!(decoded.version, SaveFormatVersion::V2);
        assert_eq!(decoded.metadata(), &metadata);
        assert_eq!(decoded.shell_state().selected_square, Some(c5));
        assert_eq!(
            decoded.shell_state().pending_promotion,
            Some(PendingPromotionSnapshot { from: e2, to: e4 })
        );
        assert_eq!(decoded.shell_state().last_move, Some(Move::new(c7, c5)));
        assert_eq!(
            decoded.shell_state().claimed_draw,
            Some(ClaimedDrawSnapshot::ThreefoldRepetition)
        );
        assert!(decoded.shell_state().dirty_recovery);
        assert_eq!(restored, after_c5);
        assert_eq!(restored.to_fen(), after_c5.to_fen());
        assert_eq!(restored.legal_moves(), after_c5.legal_moves());
        assert!(matches!(restored.status(), GameStatus::Ongoing { .. }));
    }

    #[test]
    fn recovery_snapshot_roundtrip_keeps_resume_metadata_and_shell_flags() {
        let game_state = GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1")
            .expect("fixture FEN should parse");
        let snapshot = GameSnapshot::from_parts(
            game_state.clone(),
            SnapshotMetadata {
                label: String::from("recovery"),
                created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
                updated_at_utc: Some(String::from("2026-03-15T00:01:00Z")),
                notes: Some(String::from("Interrupted session")),
                save_kind: SaveKind::Recovery,
                session_id: String::from("recovery"),
                recovery_key: Some(String::from("autosave")),
            },
            SnapshotShellState {
                dirty_recovery: true,
                ..SnapshotShellState::default()
            },
        );

        let restored = snapshot.restore_game_state();
        assert_eq!(snapshot.metadata().save_kind, SaveKind::Recovery);
        assert_eq!(
            snapshot.metadata().recovery_key.as_deref(),
            Some("autosave")
        );
        assert!(snapshot.shell_state().dirty_recovery);
        assert_eq!(restored.to_fen(), game_state.to_fen());
        assert_eq!(restored.legal_moves(), game_state.legal_moves());
    }
}
