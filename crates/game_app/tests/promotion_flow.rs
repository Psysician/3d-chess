use tempfile::tempdir;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use chess_core::{GameState, Move, Piece, PieceKind, Side, Square};
use game_app::{
    AiMatchPlugin, AppScreenState, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin,
    MatchLaunchIntent, MatchSession, MenuAction, MenuPlugin, MoveFeedbackPlugin,
    PendingLoadedSnapshot, PieceViewPlugin, PieceVisual, SaveLoadPlugin, SaveRootOverride,
    ShellInputPlugin, ShellTheme,
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

fn ui_texts(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut query = world.query::<&Text>();
    query.iter(world).map(|text| text.0.clone()).collect()
}

fn piece_visuals(app: &mut App) -> Vec<PieceVisual> {
    let world = app.world_mut();
    let mut query = world.query::<&PieceVisual>();
    query.iter(world).copied().collect()
}

#[test]
fn promotion_flow_resolves_pending_promotion_with_keyboard_choice() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    enter_local_match(&mut app);

    let from = Square::from_algebraic("a7").expect("valid square");
    let to = Square::from_algebraic("a8").expect("valid square");

    {
        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
        match_session.replace_game_state(
            GameState::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN"),
        );
        match_session.selected_square = Some(from);
        match_session.pending_promotion_move = Some(Move::new(from, to));
    }

    app.update();

    assert!(
        ui_texts(&mut app)
            .iter()
            .any(|text| text == "Choose Promotion")
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();
    {
        let mut keyboard_input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard_input.release(KeyCode::KeyQ);
        keyboard_input.clear();
    }
    app.update();

    let match_session = app.world().resource::<MatchSession>();
    assert_eq!(match_session.pending_promotion_move, None);
    assert_eq!(
        match_session.game_state().piece_at(to),
        Some(Piece::new(Side::White, PieceKind::Queen))
    );

    let piece_visuals = piece_visuals(&mut app);
    assert!(piece_visuals.iter().any(|piece_visual| {
        piece_visual.square == to && piece_visual.piece == Piece::new(Side::White, PieceKind::Queen)
    }));
}
