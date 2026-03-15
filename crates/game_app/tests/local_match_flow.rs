use tempfile::tempdir;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use chess_core::{GameState, Move, PieceKind, Side, Square};
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

fn current_state(app: &App) -> AppScreenState {
    *app.world().resource::<State<AppScreenState>>().get()
}

fn piece_visuals(app: &mut App) -> Vec<PieceVisual> {
    let world = app.world_mut();
    let mut query = world.query::<&PieceVisual>();
    query.iter(world).copied().collect()
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

#[test]
fn local_match_flow_covers_start_move_claim_draw_and_result_transition() {
    let root = tempdir().expect("temporary directory should be created");
    let mut app = test_app(root.path());
    bootstrap_shell(&mut app);
    enter_local_match(&mut app);

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(piece_visuals(&mut app).len(), 32);

    {
        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        match_session
            .apply_move(Move::new(
                Square::from_algebraic("e2").expect("valid square"),
                Square::from_algebraic("e4").expect("valid square"),
            ))
            .expect("e2e4 should be legal from the starting position");
    }

    app.update();

    let piece_visuals = piece_visuals(&mut app);
    assert_eq!(piece_visuals.len(), 32);
    assert!(piece_visuals.iter().any(|piece_visual| {
        piece_visual.square == Square::from_algebraic("e4").expect("valid square")
            && piece_visual.piece.kind == PieceKind::Pawn
            && piece_visual.piece.side == Side::White
    }));
    assert!(!piece_visuals.iter().any(|piece_visual| {
        piece_visual.square == Square::from_algebraic("e2").expect("valid square")
    }));

    {
        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
        match_session.replace_game_state(
            GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1").expect("valid FEN"),
        );
        assert!(match_session.claim_draw());
    }

    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::MatchResult);
}
