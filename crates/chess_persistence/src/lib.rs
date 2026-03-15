pub mod snapshot;

pub use snapshot::{GameSnapshot, SaveFormatVersion, SnapshotMetadata};

#[cfg(test)]
mod tests {
    use chess_core::{GameState, GameStatus, Move, Square};

    use crate::{GameSnapshot, SnapshotMetadata};

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

        let snapshot = GameSnapshot::new(
            after_c5.clone(),
            SnapshotMetadata {
                label: String::from("opening"),
                created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
                notes: Some(String::from("Persist opening state")),
            },
        );

        let encoded =
            serde_json::to_string(&snapshot).expect("serializing the snapshot should succeed");
        let decoded: GameSnapshot =
            serde_json::from_str(&encoded).expect("deserializing the snapshot should succeed");
        let restored = decoded.restore_game_state();

        assert_eq!(restored, after_c5);
        assert_eq!(restored.to_fen(), after_c5.to_fen());
        assert_eq!(restored.legal_moves(), after_c5.legal_moves());
        assert!(matches!(restored.status(), GameStatus::Ongoing { .. }));
    }
}
