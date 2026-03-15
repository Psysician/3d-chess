use std::fs;
use std::io;

use chess_core::{GameState, Move, Square};
use chess_persistence::{
    ClaimedDrawSnapshot, ConfirmActionSettings, DisplayMode, GameSnapshot,
    PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore, ShellSettings,
    SnapshotMetadata, SnapshotShellState, StoreError,
};
use tempfile::TempDir;

fn square(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn opening_snapshot(label: &str) -> GameSnapshot {
    let e2 = square("e2");
    let e4 = square("e4");
    let c7 = square("c7");
    let c5 = square("c5");
    let game_state = GameState::starting_position()
        .apply_move(Move::new(e2, e4))
        .expect("e2e4 should be legal")
        .apply_move(Move::new(c7, c5))
        .expect("c7c5 should be legal");

    GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from(label),
            created_at_utc: Some(String::from("2026-03-15T12:00:00Z")),
            updated_at_utc: None,
            notes: Some(String::from("Test fixture")),
            save_kind: SaveKind::Manual,
            session_id: String::new(),
            recovery_key: None,
        },
        SnapshotShellState {
            selected_square: Some(c5),
            pending_promotion: Some(PendingPromotionSnapshot { from: e2, to: e4 }),
            last_move: Some(Move::new(c7, c5)),
            claimed_draw: Some(ClaimedDrawSnapshot::FiftyMoveRule),
            dirty_recovery: true,
        },
    )
}

#[test]
fn manual_save_list_load_delete_roundtrip() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());

    let first = store
        .save_manual(opening_snapshot("Opening Save"))
        .expect("first manual save should succeed");
    assert_eq!(first.slot_id, "opening-save");
    assert_eq!(first.save_kind, SaveKind::Manual);

    let duplicate = store
        .save_manual(opening_snapshot("Opening Save"))
        .expect("duplicate label should allocate a distinct slot");
    assert_eq!(duplicate.slot_id, "opening-save-2");

    let mut overwrite = opening_snapshot("Opening Save");
    overwrite.metadata.session_id = first.slot_id.clone();
    overwrite.metadata.notes = Some(String::from("Overwritten"));
    let overwritten = store
        .save_manual(overwrite)
        .expect("explicit slot ids should overwrite");
    assert_eq!(overwritten.slot_id, first.slot_id);

    let loaded = store
        .load_manual(&first.slot_id)
        .expect("saved snapshot should load");
    assert_eq!(loaded.metadata().notes.as_deref(), Some("Overwritten"));
    assert_eq!(
        loaded.shell_state().pending_promotion,
        Some(PendingPromotionSnapshot {
            from: square("e2"),
            to: square("e4"),
        })
    );

    let saves = store
        .list_manual_saves()
        .expect("listing manual saves should succeed");
    assert_eq!(saves.len(), 2);
    assert!(saves.iter().any(|save| save.slot_id == "opening-save"));
    assert!(saves.iter().any(|save| save.slot_id == "opening-save-2"));

    store
        .delete_manual(&duplicate.slot_id)
        .expect("deleting a save should succeed");
    let saves = store
        .list_manual_saves()
        .expect("listing after delete should succeed");
    assert_eq!(saves.len(), 1);

    let error = store
        .load_manual(&duplicate.slot_id)
        .expect_err("deleted save should not load");
    assert!(
        matches!(error, StoreError::Io(ref io_error) if io_error.kind() == io::ErrorKind::NotFound)
    );
}

#[test]
fn recovery_record_and_settings_roundtrip() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());

    let recovery = store
        .store_recovery(opening_snapshot("Recovery"))
        .expect("recovery save should succeed");
    assert_eq!(recovery.slot_id, "recovery");
    assert_eq!(recovery.save_kind, SaveKind::Recovery);

    let loaded = store
        .load_recovery()
        .expect("recovery load should succeed")
        .expect("recovery save should exist");
    assert_eq!(loaded.metadata().save_kind, SaveKind::Recovery);
    assert_eq!(loaded.metadata().session_id, "recovery");
    assert_eq!(loaded.metadata().recovery_key.as_deref(), Some("autosave"));
    assert!(loaded.shell_state().dirty_recovery);

    let settings = ShellSettings {
        recovery_policy: RecoveryStartupPolicy::Ignore,
        confirm_actions: ConfirmActionSettings {
            overwrite_save: false,
            delete_save: true,
            abandon_match: false,
        },
        display_mode: DisplayMode::Fullscreen,
    };
    store
        .save_settings(&settings)
        .expect("settings save should succeed");
    assert_eq!(
        store.load_settings().expect("settings load should succeed"),
        settings
    );

    store
        .clear_recovery()
        .expect("recovery clear should succeed");
    assert!(
        store
            .load_recovery()
            .expect("recovery load should succeed")
            .is_none()
    );
}

#[test]
fn runtime_default_root_uses_platform_app_data_dir() {
    let root = SessionStore::default_root().expect("platform app data dir should resolve");
    assert!(root.is_absolute());
    assert_eq!(
        root.file_name().and_then(|value| value.to_str()),
        Some("3d-chess")
    );
}

#[test]
fn explicit_slot_ids_must_be_safe_path_segments() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());
    let mut snapshot = opening_snapshot("Unsafe Slot");
    snapshot.metadata.session_id = String::from("../escape");

    let error = store
        .save_manual(snapshot)
        .expect_err("path traversal slot ids should be rejected");
    assert!(matches!(error, StoreError::InvalidSlotId(_)));

    let error = store
        .load_manual("../escape")
        .expect_err("path traversal loads should be rejected");
    assert!(matches!(error, StoreError::InvalidSlotId(_)));

    let error = store
        .delete_manual("../escape")
        .expect_err("path traversal deletes should be rejected");
    assert!(matches!(error, StoreError::InvalidSlotId(_)));
}

#[test]
fn list_and_load_manual_saves_ignore_tampered_on_disk_session_ids() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());
    let mut snapshot = opening_snapshot("Tampered");
    snapshot.metadata.session_id = String::from("../escape");
    snapshot.metadata.save_kind = SaveKind::Recovery;

    let saves_dir = temp_dir.path().join("saves");
    fs::create_dir_all(&saves_dir).expect("save directory should exist");
    fs::write(
        saves_dir.join("tampered-slot.json"),
        serde_json::to_vec_pretty(&snapshot).expect("snapshot should serialize"),
    )
    .expect("tampered save should be written");

    let saves = store
        .list_manual_saves()
        .expect("listing manual saves should succeed");
    assert_eq!(saves.len(), 1);
    assert_eq!(saves[0].slot_id, "tampered-slot");
    assert_eq!(saves[0].save_kind, SaveKind::Manual);

    let loaded = store
        .load_manual("tampered-slot")
        .expect("loading tampered save should succeed");
    assert_eq!(loaded.metadata().session_id, "tampered-slot");
    assert_eq!(loaded.metadata().save_kind, SaveKind::Manual);
}

#[test]
fn corrupt_settings_file_returns_serialization_error_instead_of_defaulting() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());
    fs::write(temp_dir.path().join("settings.json"), b"{not-json")
        .expect("corrupt settings fixture should be written");

    let error = store
        .load_settings()
        .expect_err("corrupt settings should stay visible to callers");
    assert!(matches!(error, StoreError::Serialization(_)));
}

#[test]
fn corrupt_recovery_file_surfaces_error_before_resume_logic_uses_it() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());
    fs::create_dir_all(temp_dir.path().join("recovery")).expect("recovery directory should exist");
    fs::write(
        temp_dir.path().join("recovery").join("current.json"),
        b"{not-json",
    )
    .expect("corrupt recovery fixture should be written");

    let error = store
        .load_recovery()
        .expect_err("corrupt recovery data should not be hidden");
    assert!(matches!(error, StoreError::Serialization(_)));
}

#[test]
fn clear_recovery_reports_io_failures_when_path_is_a_directory() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let store = SessionStore::new(temp_dir.path());
    fs::create_dir_all(temp_dir.path().join("recovery").join("current.json"))
        .expect("directory-backed recovery fixture should exist");

    let error = store
        .clear_recovery()
        .expect_err("directory-backed recovery path should fail to clear");
    assert!(matches!(error, StoreError::Io(_)));
}
