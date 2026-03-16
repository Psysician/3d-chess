use chess_persistence::{
    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SaveKind, SavedSessionSummary,
    ShellSettings,
};
use game_app::test_support::save_load_logic;
use game_app::{RecoveryBannerState, SaveLoadState};

#[test]
fn combine_persistence_errors_joins_visible_failures_only() {
    let combined = save_load_logic::combine_persistence_errors([
        Some(String::from("save index failed")),
        None,
        Some(String::from("settings failed")),
    ]);
    assert_eq!(
        combined.as_deref(),
        Some("save index failed settings failed")
    );
}

#[test]
fn recovery_visibility_respects_ignore_policy_and_uses_cached_label() {
    let summary = SavedSessionSummary {
        slot_id: String::from("recovery"),
        label: String::from("Interrupted Session"),
        created_at_utc: None,
        save_kind: SaveKind::Recovery,
    };
    let mut save_state = SaveLoadState {
        recovery: Some(summary.clone()),
        settings: ShellSettings {
            recovery_policy: RecoveryStartupPolicy::Ignore,
            confirm_actions: ConfirmActionSettings::default(),
            display_mode: DisplayMode::Windowed,
        },
        ..Default::default()
    };
    let mut banner = RecoveryBannerState::default();

    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut banner);
    assert!(!banner.available);
    assert_eq!(banner.label, None);

    save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
    save_load_logic::sync_cached_recovery_visibility(&save_state, &mut banner);
    assert!(banner.available);
    assert_eq!(banner.label.as_deref(), Some("Interrupted Session"));
    assert_eq!(
        save_load_logic::recovery_banner_label(Some(&summary)).as_deref(),
        Some("Interrupted Session")
    );
}

#[test]
fn save_feedback_messages_and_policy_copy_stay_deterministic() {
    let summary = SavedSessionSummary {
        slot_id: String::from("slot-a"),
        label: String::from("Slot A"),
        created_at_utc: None,
        save_kind: SaveKind::Manual,
    };
    assert_eq!(
        save_load_logic::manual_save_message(&summary),
        "Saved match as Slot A."
    );
    assert_eq!(
        save_load_logic::deleted_save_message("slot-a"),
        "Deleted save slot-a."
    );
    assert!(
        save_load_logic::recovery_policy_status_copy(RecoveryStartupPolicy::Resume)
            .contains("MatchLoading")
    );
}
