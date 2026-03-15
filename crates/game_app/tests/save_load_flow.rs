use std::fs;

use chess_persistence::{
    DisplayMode, GameSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore, ShellSettings,
    SnapshotMetadata, SnapshotShellState,
};
use tempfile::tempdir;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use chess_core::{GameState, Move, PieceKind, Side, Square};
use game_app::{
    AiMatchPlugin, AppScreenState, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin,
    MatchLaunchIntent, MatchSession, MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin,
    PendingLoadedSnapshot, PieceViewPlugin, PieceVisual, RecoveryBannerState, SaveLoadPlugin,
    SaveLoadRequest, SaveLoadState, SaveRootOverride, ShellInputPlugin, ShellMenuState, ShellTheme,
};

fn test_app(root: &std::path::Path) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .insert_resource(Assets::<Mesh>::default())
        .insert_resource(Assets::<StandardMaterial>::default())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ShellTheme::default())
        .insert_resource(MatchSession::start_local_match())
        .insert_resource(MatchLaunchIntent::default())
        .insert_resource(PendingLoadedSnapshot::default())
        .insert_resource(SaveRootOverride(Some(root.to_path_buf())))
        .init_state::<AppScreenState>()
        .add_plugins((
            MenuPlugin,
            SaveLoadPlugin,
            AppShellPlugin,
            BoardScenePlugin,
            PieceViewPlugin,
            ShellInputPlugin,
            MoveFeedbackPlugin,
            AiMatchPlugin,
            ChessAudioPlugin,
        ));
    app
}

fn bootstrap_shell(app: &mut App) {
    app.update();
    app.update();
}

fn enter_local_match(app: &mut App) {
    app.world_mut().write_message(MenuAction::OpenSetup);
    app.update();
    app.world_mut().write_message(MenuAction::StartNewMatch);
    app.update();
    app.update();
    app.update();
}

fn current_state(app: &App) -> AppScreenState {
    *app.world().resource::<State<AppScreenState>>().get()
}

fn tap_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
}

fn piece_visuals(app: &mut App) -> Vec<PieceVisual> {
    let world = app.world_mut();
    let mut query = world.query::<&PieceVisual>();
    query.iter(world).copied().collect()
}

fn recovery_snapshot(label: &str) -> GameSnapshot {
    GameSnapshot::from_parts(
        GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").expect("fixture FEN should parse"),
        SnapshotMetadata {
            label: String::from(label),
            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
            updated_at_utc: None,
            notes: Some(String::from("Recovery fixture")),
            save_kind: SaveKind::Recovery,
            session_id: String::from("recovery"),
            recovery_key: Some(String::from("autosave")),
        },
        SnapshotShellState::default(),
    )
}

#[test]
fn manual_save_and_load_roundtrip_restores_pending_promotion() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    enter_local_match(&mut app);

    let promotion_from = Square::from_algebraic("e7").expect("valid square");
    let promotion_to = Square::from_algebraic("e8").expect("valid square");
    {
        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
        match_session.replace_game_state(
            GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1")
                .expect("fixture FEN should parse"),
        );
        match_session.selected_square = Some(promotion_from);
        match_session.pending_promotion_move = Some(Move::new(promotion_from, promotion_to));
        match_session.mark_recovery_dirty();
    }
    app.update();

    app.world_mut().write_message(SaveLoadRequest::SaveManual {
        label: String::from("Promotion Save"),
        slot_id: None,
    });
    app.update();
    app.update();
    assert_eq!(
        app.world().resource::<SaveLoadState>().manual_saves.len(),
        1
    );

    let slot_id = app.world().resource::<SaveLoadState>().manual_saves[0]
        .slot_id
        .clone();

    app.world_mut()
        .resource_mut::<MatchSession>()
        .reset_for_local_match();
    app.update();

    app.world_mut()
        .write_message(SaveLoadRequest::LoadManual { slot_id });
    app.update();
    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    let match_session = app.world().resource::<MatchSession>();
    assert_eq!(
        match_session.pending_promotion_move,
        Some(Move::new(promotion_from, promotion_to))
    );
    assert_eq!(
        match_session.game_state,
        GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("fixture FEN should parse")
    );

    let piece_visuals = piece_visuals(&mut app);
    assert!(piece_visuals.iter().any(|piece_visual| {
        piece_visual.square == promotion_from
            && piece_visual.piece.kind == PieceKind::Pawn
            && piece_visual.piece.side == Side::White
    }));
}

#[test]
fn quick_save_stays_available_in_match_but_not_through_pause_overlay() {
    let root = tempdir().expect("temporary directory should be created");
    let expected_state =
        GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").expect("fixture FEN should parse");

    {
        let mut app = test_app(root.path());
        bootstrap_shell(&mut app);
        enter_local_match(&mut app);

        {
            let mut match_session = app.world_mut().resource_mut::<MatchSession>();
            match_session.replace_game_state(expected_state.clone());
            match_session.mark_recovery_dirty();
        }
        app.update();

        tap_key(&mut app, KeyCode::F5);
        assert_eq!(
            app.world().resource::<SaveLoadState>().manual_saves.len(),
            1
        );

        tap_key(&mut app, KeyCode::Escape);
        assert_eq!(current_state(&app), AppScreenState::InMatch);
        assert_eq!(
            app.world().resource::<ShellMenuState>().panel,
            MenuPanel::Setup
        );

        tap_key(&mut app, KeyCode::F5);
        assert_eq!(
            app.world().resource::<SaveLoadState>().manual_saves.len(),
            1
        );

        {
            let mut save_state = app.world_mut().resource_mut::<SaveLoadState>();
            save_state.settings.display_mode = DisplayMode::Fullscreen;
            save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
        }
        app.world_mut()
            .write_message(SaveLoadRequest::PersistSettings);
        app.update();
    }

    let mut restarted = test_app(root.path());
    bootstrap_shell(&mut restarted);

    let save_state = restarted.world().resource::<SaveLoadState>();
    assert_eq!(save_state.settings.display_mode, DisplayMode::Fullscreen);
    assert_eq!(
        save_state.settings.recovery_policy,
        RecoveryStartupPolicy::Ask
    );
    assert!(
        restarted
            .world()
            .resource::<RecoveryBannerState>()
            .available
    );
    assert_eq!(save_state.manual_saves.len(), 1);

    restarted
        .world_mut()
        .write_message(SaveLoadRequest::ResumeRecovery);
    restarted.update();
    restarted.update();
    restarted.update();

    assert_eq!(current_state(&restarted), AppScreenState::InMatch);
    let match_session = restarted.world().resource::<MatchSession>();
    assert_eq!(match_session.game_state, expected_state);
}

#[test]
fn ignore_policy_hides_recovery_banner_across_refreshes() {
    let root = tempdir().expect("temporary directory should be created");
    let store = SessionStore::new(root.path());
    store
        .store_recovery(recovery_snapshot("Recovery Fixture"))
        .expect("recovery fixture should be written");
    store
        .save_settings(&ShellSettings {
            recovery_policy: RecoveryStartupPolicy::Ignore,
            ..ShellSettings::default()
        })
        .expect("settings should be written");

    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);

    {
        let recovery = app.world().resource::<RecoveryBannerState>();
        assert!(!recovery.available);
        assert_eq!(recovery.label, None);
    }

    app.world_mut().write_message(SaveLoadRequest::RefreshIndex);
    app.update();

    let recovery = app.world().resource::<RecoveryBannerState>();
    assert!(!recovery.available);
    assert_eq!(recovery.label, None);
}

#[test]
fn abandon_request_keeps_live_match_when_recovery_clear_fails() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    enter_local_match(&mut app);

    let bad_recovery_path = root.path().join("recovery").join("current.json");
    fs::create_dir_all(&bad_recovery_path).expect("directory-backed recovery path should exist");

    app.world_mut()
        .write_message(SaveLoadRequest::AbandonMatchAndReturnToMenu);
    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(
        app.world()
            .resource::<SaveLoadState>()
            .last_error
            .as_deref(),
        Some("Unable to clear interrupted-session recovery.")
    );
}

#[test]
fn entering_match_result_clears_recovery_label_cache() {
    let root = tempdir().expect("temporary directory should be created");
    let store = SessionStore::new(root.path());
    store
        .store_recovery(recovery_snapshot("Recovery Fixture"))
        .expect("recovery fixture should be written");

    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);

    assert_eq!(
        app.world()
            .resource::<RecoveryBannerState>()
            .label
            .as_deref(),
        Some("Recovery Fixture")
    );

    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MatchResult);
    app.update();
    app.update();

    let recovery = app.world().resource::<RecoveryBannerState>();
    assert!(!recovery.available);
    assert_eq!(recovery.label, None);
    assert!(
        store
            .load_recovery()
            .expect("recovery load should succeed")
            .is_none()
    );
}
