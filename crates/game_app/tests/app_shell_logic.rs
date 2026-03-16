use chess_core::{
    AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Move, Side, Square, WinReason,
};
use chess_persistence::{
    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SaveKind, SavedSessionSummary,
    ShellSettings,
};
use game_app::test_support::app_shell_logic;
use game_app::{
    AppScreenState, ClaimedDrawReason, ConfirmationKind, MenuContext, RecoveryBannerState,
    SaveLoadState, ShellMenuState,
};

#[test]
fn status_priority_prefers_errors_messages_then_recovery_copy() {
    let menu_state = ShellMenuState {
        status_line: Some(String::from("menu status")),
        ..Default::default()
    };
    let save_state = SaveLoadState {
        last_error: Some(String::from("save error")),
        last_message: Some(String::from("save message")),
        ..Default::default()
    };
    let recovery = RecoveryBannerState {
        available: true,
        dirty: false,
        label: Some(String::from("Interrupted Session")),
    };

    assert_eq!(
        app_shell_logic::effective_shell_status(&menu_state, &save_state, &recovery).as_deref(),
        Some("save error")
    );

    let save_state = SaveLoadState {
        last_message: Some(String::from("save message")),
        ..Default::default()
    };
    assert_eq!(
        app_shell_logic::effective_shell_status(&menu_state, &save_state, &recovery).as_deref(),
        Some("save message")
    );

    let save_state = SaveLoadState::default();
    assert_eq!(
        app_shell_logic::effective_shell_status(&menu_state, &save_state, &recovery).as_deref(),
        Some("menu status")
    );

    let menu_state = ShellMenuState::default();
    assert_eq!(
        app_shell_logic::effective_shell_status(&menu_state, &save_state, &recovery).as_deref(),
        Some("Interrupted-session recovery is available as Interrupted Session.")
    );
}

#[test]
fn labels_and_policy_cycles_cover_setup_copy_branches() {
    assert_eq!(
        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Resume),
        RecoveryStartupPolicy::Ask
    );
    assert_eq!(
        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Ask),
        RecoveryStartupPolicy::Ignore
    );
    assert_eq!(
        app_shell_logic::next_recovery_policy(RecoveryStartupPolicy::Ignore),
        RecoveryStartupPolicy::Resume
    );
    assert_eq!(
        app_shell_logic::recovery_policy_label(RecoveryStartupPolicy::Resume),
        "Resume automatically"
    );
    assert_eq!(
        app_shell_logic::display_mode_label(DisplayMode::Fullscreen),
        "Fullscreen"
    );
    assert_eq!(
        app_shell_logic::toggle_label("Overwrite Save", true),
        "Overwrite Save: on"
    );
    assert_eq!(
        app_shell_logic::confirmation_copy(ConfirmationKind::DeleteSave).0,
        "Delete the selected save?"
    );
}

#[test]
fn result_copy_covers_checkmate_claimed_draw_and_selected_save_lookup() {
    let white_checkmate = GameStatus::Finished(GameOutcome::Win {
        winner: Side::White,
        reason: WinReason::Checkmate,
    });
    assert_eq!(
        app_shell_logic::match_session_result_title(white_checkmate, None),
        "White Wins"
    );
    assert_eq!(
        app_shell_logic::match_session_result_detail(white_checkmate, None),
        "Checkmate detected by chess_core."
    );

    let last_move = Some(Move::new(
        Square::from_algebraic("e2").expect("fixture square should parse"),
        Square::from_algebraic("e4").expect("fixture square should parse"),
    ));
    assert_eq!(
        app_shell_logic::derive_save_label(last_move),
        "Local Match after e2e4"
    );
    assert_eq!(app_shell_logic::derive_save_label(None), "Local Match Save");

    let fivefold = GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
        AutomaticDrawReason::FivefoldRepetition,
    )));
    assert_eq!(
        app_shell_logic::match_session_result_title(
            fivefold,
            Some(ClaimedDrawReason::ThreefoldRepetition),
        ),
        "Draw Claimed by Repetition"
    );

    let save_state = SaveLoadState {
        manual_saves: vec![SavedSessionSummary {
            slot_id: String::from("slot-a"),
            label: String::from("Slot A"),
            created_at_utc: None,
            save_kind: SaveKind::Manual,
        }],
        settings: ShellSettings {
            recovery_policy: RecoveryStartupPolicy::Ask,
            confirm_actions: ConfirmActionSettings::default(),
            display_mode: DisplayMode::Windowed,
        },
        ..Default::default()
    };
    let menu_state = ShellMenuState {
        context: MenuContext::InMatchOverlay,
        selected_save: Some(String::from("slot-a")),
        ..Default::default()
    };
    assert!(app_shell_logic::return_to_menu_abandons_active_match(
        AppScreenState::InMatch,
        &menu_state,
    ));
    assert_eq!(
        app_shell_logic::selected_save_summary(&menu_state, &save_state)
            .map(|summary| summary.label.as_str()),
        Some("Slot A")
    );
}
