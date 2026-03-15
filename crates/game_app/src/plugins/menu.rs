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
