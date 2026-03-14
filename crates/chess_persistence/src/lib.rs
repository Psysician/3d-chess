pub mod snapshot;

pub use snapshot::{GameSnapshot, SaveFormatVersion, SnapshotMetadata};

#[cfg(test)]
mod tests {
    use chess_core::GameState;

    use crate::GameSnapshot;

    #[test]
    fn placeholder_snapshot_roundtrips_through_json() {
        let snapshot = GameSnapshot::placeholder(GameState::default());
        let encoded = serde_json::to_string(&snapshot)
            .expect("serializing the placeholder snapshot should succeed");
        let decoded: GameSnapshot = serde_json::from_str(&encoded)
            .expect("deserializing the placeholder snapshot should succeed");

        assert_eq!(decoded, snapshot);
    }
}
