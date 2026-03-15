use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use chess_core::{GameState, Square};
use game_app::{AppScreenState, AppShellPlugin, MatchSession, ShellTheme};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .insert_resource(ShellTheme::default())
        .insert_resource(MatchSession::start_local_match())
        .init_state::<AppScreenState>()
        .add_plugins(AppShellPlugin);
    app
}

fn current_state(app: &App) -> AppScreenState {
    *app.world().resource::<State<AppScreenState>>().get()
}

#[test]
fn match_loading_resets_session_and_enters_in_match() {
    let mut app = test_app();

    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::MainMenu);

    {
        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        match_session.replace_game_state(
            GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").expect("valid FEN"),
        );
    }

    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MatchLoading);

    app.update();
    app.update();

    let match_session = app.world().resource::<MatchSession>();
    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(match_session.game_state, GameState::starting_position());
    assert_eq!(match_session.selected_square, None);
    assert_eq!(match_session.pending_promotion_move, None);
}

#[test]
fn finished_match_reaches_result_then_supports_rematch_and_menu_return() {
    let mut app = test_app();

    app.update();
    app.update();

    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MatchLoading);

    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);

    app.world_mut()
        .resource_mut::<MatchSession>()
        .replace_game_state(
            GameState::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN"),
        );

    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::MatchResult);

    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MatchLoading);

    app.update();
    app.update();

    assert_eq!(current_state(&app), AppScreenState::InMatch);
    assert_eq!(
        app.world().resource::<MatchSession>().game_state,
        GameState::starting_position()
    );

    app.world_mut()
        .resource_mut::<NextState<AppScreenState>>()
        .set(AppScreenState::MainMenu);

    app.update();

    assert_eq!(current_state(&app), AppScreenState::MainMenu);
}
