use chess_persistence::{
    GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore,
    ShellSettings, SnapshotMetadata, SnapshotShellState,
};
use tempfile::tempdir;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use chess_core::{GameState, Move, Square};
use game_app::{
    AiMatchPlugin, AppScreenState, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin,
    MatchLaunchIntent, MatchSession, MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin,
    PendingLoadedSnapshot, PieceViewPlugin, SaveLoadPlugin, SaveRootOverride, ShellInputPlugin,
    ShellMenuState, ShellTheme,
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
}

fn sample_snapshot(label: &str) -> GameSnapshot {
    let game_state =
        GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("fixture FEN should parse");
    let from = Square::from_algebraic("e7").expect("valid square");
    let to = Square::from_algebraic("e8").expect("valid square");

    GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: label.to_string(),
            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Manual,
            session_id: label.to_ascii_lowercase().replace(' ', "-"),
            recovery_key: None,
        },
        SnapshotShellState {
            selected_square: Some(from),
            pending_promotion: Some(PendingPromotionSnapshot { from, to }),
            last_move: Some(Move::new(from, to)),
            claimed_draw: None,
            dirty_recovery: true,
        },
    )
}

#[test]
fn manual_load_intent_restores_snapshot_and_enters_in_match() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);

    *app.world_mut().resource_mut::<MatchLaunchIntent>() = MatchLaunchIntent::LoadManual;
    app.world_mut().resource_mut::<PendingLoadedSnapshot>().0 =
        Some(sample_snapshot("Manual Fixture"));
    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MatchLoading);

    app.update();
    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    let match_session = app.world().resource::<MatchSession>();
    assert_eq!(
        match_session.pending_promotion_move,
        Some(Move::new(
            Square::from_algebraic("e7").expect("valid square"),
            Square::from_algebraic("e8").expect("valid square"),
        ))
    );
}

#[test]
fn escape_opens_setup_overlay_without_leaving_in_match_state() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    enter_local_match(&mut app);

    tap_key(&mut app, KeyCode::Escape);

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(
        app.world().resource::<ShellMenuState>().panel,
        MenuPanel::Setup
    );

    app.world_mut().write_message(MenuAction::ReturnToMenu);
    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::MainMenu);
}

#[test]
fn startup_resume_policy_hydrates_recovery_snapshot_through_match_loading() {
    let root = tempdir().expect("temporary directory should be created");
    let store = SessionStore::new(root.path());
    store
        .store_recovery(sample_snapshot("Recovery Fixture"))
        .expect("recovery save should succeed");
    store
        .save_settings(&ShellSettings {
            recovery_policy: RecoveryStartupPolicy::Resume,
            ..ShellSettings::default()
        })
        .expect("settings save should succeed");

    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    app.update();
    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(
        app.world().resource::<MatchSession>().selected_square,
        Some(Square::from_algebraic("e7").expect("valid square"))
    );
}
