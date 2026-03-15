use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};

use crate::match_state::MatchSession;
use crate::plugins::{
    AiMatchPlugin, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin, MenuPlugin,
    MoveFeedbackPlugin, PieceViewPlugin, SaveLoadPlugin, ShellInputPlugin,
};
use crate::style::ShellTheme;

pub const APP_TITLE: &str = "3D Chess";

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppScreenState {
    #[default]
    Boot,
    MainMenu,
    LocalSetup,
    MatchLoading,
    InMatch,
    Paused,
    MatchResult,
}

#[must_use]
pub fn build_app() -> App {
    let shell_theme = ShellTheme::default();
    let mut app = App::new();

    app.insert_resource(ClearColor(shell_theme.clear_color))
        .insert_resource(shell_theme)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from(APP_TITLE),
                resolution: WindowResolution::new(1600, 900),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppScreenState>()
        .insert_resource(MatchSession::start_local_match())
        .add_plugins((
            AppShellPlugin,
            BoardScenePlugin,
            PieceViewPlugin,
            ShellInputPlugin,
            MoveFeedbackPlugin,
            MenuPlugin,
            SaveLoadPlugin,
            AiMatchPlugin,
            ChessAudioPlugin,
        ));

    app
}

pub fn run() {
    build_app().run();
}
