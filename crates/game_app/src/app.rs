use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};

use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
use crate::plugins::{
    AiMatchPlugin, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin, MenuPlugin,
    MoveFeedbackPlugin, PieceViewPlugin, RecoveryBannerState, SaveLoadPlugin, SaveLoadState,
    SaveRootOverride, ShellInputPlugin, ShellMenuState,
};
use crate::style::ShellTheme;

pub const APP_TITLE: &str = "3D Chess";

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppScreenState {
    #[default]
    Boot,
    MainMenu,
    MatchLoading,
    InMatch,
    MatchResult,
}

/// Builds the coarse screen-state shell and keeps menu/save-load concerns in orthogonal resources.
/// Modal flow stays outside the top-level route enum so the local shell grows without routing sprawl. (ref: DL-001) (ref: DL-007)
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
        // These resources carry launch intent and modal shell state across the small set of top-level routes. (ref: DL-001)
        .insert_resource(MatchLaunchIntent::default())
        .insert_resource(PendingLoadedSnapshot::default())
        .insert_resource(ShellMenuState::default())
        .insert_resource(RecoveryBannerState::default())
        .insert_resource(SaveLoadState::default())
        .insert_resource(SaveRootOverride::default())
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

pub fn run() {
    build_app().run();
}
