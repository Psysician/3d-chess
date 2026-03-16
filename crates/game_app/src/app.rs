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

    install_shell_resources(&mut app, shell_theme)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window()),
            ..default()
        }))
        .init_state::<AppScreenState>();
    install_shell_plugins(&mut app);

    app
}

pub fn run() {
    build_app().run();
}

fn primary_window() -> Window {
    Window {
        title: String::from(APP_TITLE),
        resolution: WindowResolution::new(1600, 900),
        present_mode: PresentMode::AutoVsync,
        resizable: true,
        ..default()
    }
}

fn install_shell_resources(app: &mut App, shell_theme: ShellTheme) -> &mut App {
    app.insert_resource(ClearColor(shell_theme.clear_color))
        .insert_resource(shell_theme)
        .insert_resource(MatchSession::start_local_match())
        .insert_resource(MatchLaunchIntent::default())
        .insert_resource(PendingLoadedSnapshot::default())
        .insert_resource(ShellMenuState::default())
        .insert_resource(RecoveryBannerState::default())
        .insert_resource(SaveLoadState::default())
        .insert_resource(SaveRootOverride::default())
}

fn install_shell_plugins(app: &mut App) -> &mut App {
    app.add_plugins((
        MenuPlugin,
        SaveLoadPlugin,
        AppShellPlugin,
        BoardScenePlugin,
        PieceViewPlugin,
        ShellInputPlugin,
        MoveFeedbackPlugin,
        AiMatchPlugin,
        ChessAudioPlugin,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn helpers_configure_window_and_shell_resources() {
        let window = primary_window();
        assert_eq!(window.title, APP_TITLE);
        assert_eq!(window.present_mode, PresentMode::AutoVsync);
        assert!(window.resizable);

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppScreenState>();
        install_shell_resources(&mut app, ShellTheme::default());

        assert_eq!(
            app.world().resource::<State<AppScreenState>>().get(),
            &AppScreenState::Boot
        );
        assert_eq!(
            app.world().resource::<ClearColor>().0,
            app.world().resource::<ShellTheme>().clear_color
        );
        assert!(
            app.world()
                .resource::<MatchSession>()
                .summary()
                .dirty_recovery
        );
        assert_eq!(
            app.world().resource::<MatchLaunchIntent>(),
            &MatchLaunchIntent::default()
        );
        assert_eq!(app.world().resource::<PendingLoadedSnapshot>().0, None);
        assert_eq!(
            app.world().resource::<ShellMenuState>(),
            &ShellMenuState::default()
        );
        assert_eq!(
            app.world().resource::<RecoveryBannerState>(),
            &RecoveryBannerState::default()
        );
        let save_state = app.world().resource::<SaveLoadState>();
        assert!(save_state.manual_saves.is_empty());
        assert_eq!(save_state.recovery, None);
        assert_eq!(save_state.last_message, None);
        assert_eq!(save_state.last_error, None);
        assert_eq!(app.world().resource::<SaveRootOverride>().0, None);
    }
}
