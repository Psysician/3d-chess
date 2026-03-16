use bevy::prelude::*;

use crate::app::AppScreenState;
use crate::match_state::MatchLaunchIntent;

pub struct MenuPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPanel {
    #[default]
    Home,
    Setup,
    LoadList,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuContext {
    #[default]
    MainMenu,
    InMatchOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationKind {
    AbandonMatch,
    DeleteSave,
    OverwriteSave,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryBannerState {
    pub available: bool,
    pub dirty: bool,
    pub label: Option<String>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct ShellMenuState {
    pub panel: MenuPanel,
    pub context: MenuContext,
    pub confirmation: Option<ConfirmationKind>,
    pub selected_save: Option<String>,
    pub status_line: Option<String>,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    OpenSetup,
    OpenLoadList,
    OpenSettings,
    BackToSetup,
    StartNewMatch,
    Rematch,
    PauseMatch,
    ResumeMatch,
    ReturnToMenu,
    SelectSave(String),
    RequestConfirmation(ConfirmationKind),
    CancelModal,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShellMenuState>()
            .init_resource::<RecoveryBannerState>()
            .add_message::<MenuAction>()
            .add_systems(Update, sync_menu_panel_from_screen)
            .add_systems(Update, apply_menu_actions);
    }
}

fn sync_menu_panel_from_screen(
    state: Res<State<AppScreenState>>,
    mut menu_state: ResMut<ShellMenuState>,
) {
    if !state.is_changed() {
        return;
    }

    match state.get() {
        AppScreenState::MainMenu => {
            menu_state.context = MenuContext::MainMenu;
            menu_state.panel = MenuPanel::Home;
            menu_state.confirmation = None;
        }
        AppScreenState::MatchLoading => {
            menu_state.confirmation = None;
        }
        AppScreenState::Boot | AppScreenState::InMatch | AppScreenState::MatchResult => {}
    }
}

fn apply_menu_actions(
    mut actions: MessageReader<MenuAction>,
    state: Res<State<AppScreenState>>,
    mut menu_state: ResMut<ShellMenuState>,
    mut launch_intent: ResMut<MatchLaunchIntent>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    for action in actions.read() {
        match action {
            MenuAction::OpenSetup => {
                menu_state.panel = MenuPanel::Setup;
                menu_state.context = MenuContext::MainMenu;
            }
            MenuAction::OpenLoadList => {
                menu_state.panel = MenuPanel::LoadList;
            }
            MenuAction::OpenSettings => {
                menu_state.panel = MenuPanel::Settings;
            }
            MenuAction::BackToSetup => {
                menu_state.panel = MenuPanel::Setup;
            }
            MenuAction::StartNewMatch => {
                *launch_intent = MatchLaunchIntent::NewLocalMatch;
                menu_state.context = MenuContext::MainMenu;
                next_state.set(AppScreenState::MatchLoading);
            }
            MenuAction::Rematch => {
                *launch_intent = MatchLaunchIntent::Rematch;
                menu_state.context = MenuContext::MainMenu;
                next_state.set(AppScreenState::MatchLoading);
            }
            MenuAction::PauseMatch => {
                if *state.get() == AppScreenState::InMatch {
                    menu_state.panel = MenuPanel::Setup;
                    menu_state.context = MenuContext::InMatchOverlay;
                    menu_state.confirmation = None;
                }
            }
            MenuAction::ResumeMatch => {
                menu_state.confirmation = None;
                menu_state.context = MenuContext::MainMenu;
            }
            MenuAction::ReturnToMenu => {
                menu_state.panel = MenuPanel::Home;
                menu_state.context = MenuContext::MainMenu;
                menu_state.confirmation = None;
                next_state.set(AppScreenState::MainMenu);
            }
            MenuAction::SelectSave(slot_id) => {
                menu_state.selected_save = Some(slot_id.clone());
                menu_state.status_line = Some(format!("Selected save {slot_id}."));
            }
            MenuAction::RequestConfirmation(kind) => {
                menu_state.confirmation = Some(*kind);
            }
            MenuAction::CancelModal => {
                menu_state.confirmation = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    fn menu_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .insert_resource(MatchLaunchIntent::default())
            .init_state::<AppScreenState>()
            .add_plugins(MenuPlugin);
        app
    }

    #[test]
    fn screen_sync_resets_menu_when_entering_main_menu() {
        let mut app = menu_app();
        app.world_mut()
            .resource_mut::<NextState<AppScreenState>>()
            .set(AppScreenState::MainMenu);
        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<ShellMenuState>(),
            &ShellMenuState::default()
        );
    }

    #[test]
    fn actions_cover_panel_navigation_selection_and_confirmations() {
        let mut app = menu_app();

        app.world_mut().write_message(MenuAction::OpenSetup);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().panel,
            MenuPanel::Setup
        );

        app.world_mut().write_message(MenuAction::OpenLoadList);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().panel,
            MenuPanel::LoadList
        );

        app.world_mut().write_message(MenuAction::OpenSettings);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().panel,
            MenuPanel::Settings
        );

        app.world_mut().write_message(MenuAction::BackToSetup);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().panel,
            MenuPanel::Setup
        );

        app.world_mut()
            .write_message(MenuAction::SelectSave(String::from("slot-a")));
        app.update();
        assert_eq!(
            app.world()
                .resource::<ShellMenuState>()
                .selected_save
                .as_deref(),
            Some("slot-a")
        );
        assert_eq!(
            app.world()
                .resource::<ShellMenuState>()
                .status_line
                .as_deref(),
            Some("Selected save slot-a.")
        );

        app.world_mut()
            .write_message(MenuAction::RequestConfirmation(
                ConfirmationKind::DeleteSave,
            ));
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().confirmation,
            Some(ConfirmationKind::DeleteSave)
        );

        app.world_mut().write_message(MenuAction::CancelModal);
        app.update();
        assert_eq!(app.world().resource::<ShellMenuState>().confirmation, None);
    }

    #[test]
    fn actions_cover_match_loading_pause_resume_and_return() {
        let mut app = menu_app();

        app.world_mut().write_message(MenuAction::StartNewMatch);
        app.update();
        assert_eq!(
            *app.world().resource::<MatchLaunchIntent>(),
            MatchLaunchIntent::NewLocalMatch
        );

        app.world_mut()
            .resource_mut::<NextState<AppScreenState>>()
            .set(AppScreenState::InMatch);
        app.update();
        app.update();

        app.world_mut().write_message(MenuAction::PauseMatch);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().context,
            MenuContext::InMatchOverlay
        );

        app.world_mut().write_message(MenuAction::ResumeMatch);
        app.update();
        assert_eq!(
            app.world().resource::<ShellMenuState>().context,
            MenuContext::MainMenu
        );

        app.world_mut().write_message(MenuAction::Rematch);
        app.update();
        assert_eq!(
            *app.world().resource::<MatchLaunchIntent>(),
            MatchLaunchIntent::Rematch
        );

        app.world_mut().write_message(MenuAction::ReturnToMenu);
        app.update();
        app.update();
        assert_eq!(
            *app.world().resource::<State<AppScreenState>>().get(),
            AppScreenState::MainMenu
        );
    }
}
