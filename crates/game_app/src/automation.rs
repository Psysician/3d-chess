// Semantic automation contract for `game_app`.
// Commands and snapshots stay at the shell and match boundary so agents and
// optional adapters reuse player-visible semantics. (refs: DL-001, DL-002)

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use bevy::prelude::{App, World};
use chess_core::{DrawAvailability, GameStatus, Move, PieceKind, Square};
use chess_persistence::{SavedSessionSummary, ShellSettings};

#[cfg(feature = "automation-transport")]
use serde::{Deserialize, Serialize};

use crate::app::{build_headless_app, AppScreenState};
use crate::match_state::{ClaimedDrawReason, MatchSession};
use crate::plugins::app_shell_logic;
use crate::plugins::{
    MenuContext, MenuPanel, RecoveryBannerState, SaveLoadState, ShellMenuState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationScreen {
    #[default]
    Boot,
    MainMenu,
    MatchLoading,
    InMatch,
    MatchResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationMenuPanel {
    #[default]
    Home,
    Setup,
    LoadList,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationMenuContext {
    #[default]
    MainMenu,
    InMatchOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationConfirmationKind {
    AbandonMatch,
    DeleteSave,
    OverwriteSave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationClaimedDrawReason {
    ThreefoldRepetition,
    FiftyMoveRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(tag = "type", content = "action", rename_all = "snake_case")
)]
pub enum AutomationCommand {
    Snapshot,
    Step { frames: u32 },
    Navigation(AutomationNavigationAction),
    Save(AutomationSaveAction),
    Settings(AutomationSettingsAction),
    Match(AutomationMatchAction),
    Confirm(AutomationConfirmationKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum AutomationNavigationAction {
    OpenSetup,
    BackToSetup,
    OpenLoadList,
    OpenSettings,
    StartNewMatch,
    ResumeRecovery,
    PauseMatch,
    ResumeMatch,
    ReturnToMenu,
    Rematch,
    CancelModal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum AutomationSaveAction {
    RefreshIndex,
    SaveManual { label: Option<String> },
    SelectSlot { slot_id: String },
    LoadSelected,
    DeleteSelected,
    OverwriteSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum AutomationSettingsAction {
    CycleRecoveryPolicy,
    ToggleDisplayMode,
    ToggleConfirmation { kind: AutomationConfirmationKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum AutomationMatchAction {
    SelectSquare { square: Square },
    SubmitMove {
        from: Square,
        to: Square,
        promotion: Option<PieceKind>,
    },
    ChoosePromotion { piece: PieceKind },
    ClearInteraction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationError {
    InvalidStepCount(u32),
    SaveSelectionRequired,
    PromotionUnavailable,
    CommandIgnored(String),
}

pub type AutomationResult<T> = Result<T, AutomationError>;

impl Display for AutomationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStepCount(frames) => {
                write!(
                    formatter,
                    "automation step count must be at least one frame, got {frames}"
                )
            }
            Self::SaveSelectionRequired => {
                formatter.write_str("automation command requires a selected save slot")
            }
            Self::PromotionUnavailable => {
                formatter.write_str("automation command requires a pending promotion move")
            }
            Self::CommandIgnored(reason) => {
                write!(formatter, "automation command had no effect: {reason}")
            }
        }
    }
}

impl std::error::Error for AutomationError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize)
)]
pub struct AutomationMenuSnapshot {
    pub panel: AutomationMenuPanel,
    pub context: AutomationMenuContext,
    pub confirmation: Option<AutomationConfirmationKind>,
    pub selected_save: Option<String>,
    pub status_line: Option<String>,
    pub shell_status: Option<String>,
    pub recovery_available: bool,
    pub recovery_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize)
)]
pub struct AutomationMatchSnapshot {
    pub fen: String,
    pub status: GameStatus,
    pub selected_square: Option<Square>,
    pub legal_targets: Vec<Square>,
    pub pending_promotion: Option<Move>,
    pub last_move: Option<Move>,
    pub claimable_draw: DrawAvailability,
    pub claimed_draw: Option<AutomationClaimedDrawReason>,
    pub dirty_recovery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize)
)]
pub struct AutomationSaveSnapshot {
    pub manual_saves: Vec<SavedSessionSummary>,
    pub recovery: Option<SavedSessionSummary>,
    pub settings: ShellSettings,
    pub last_message: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automation-transport",
    derive(Serialize, Deserialize)
)]
pub struct AutomationSnapshot {
    pub screen: AutomationScreen,
    pub menu: AutomationMenuSnapshot,
    pub match_state: AutomationMatchSnapshot,
    pub saves: AutomationSaveSnapshot,
}

impl Default for AutomationMatchSnapshot {
    fn default() -> Self {
        Self {
            fen: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            status: GameStatus::Ongoing {
                in_check: false,
                draw_available: DrawAvailability::default(),
            },
            selected_square: None,
            legal_targets: Vec::new(),
            pending_promotion: None,
            last_move: None,
            claimable_draw: DrawAvailability::default(),
            claimed_draw: None,
            dirty_recovery: false,
        }
    }
}

impl Default for AutomationSnapshot {
    fn default() -> Self {
        Self {
            screen: AutomationScreen::Boot,
            menu: AutomationMenuSnapshot::default(),
            match_state: AutomationMatchSnapshot::default(),
            saves: AutomationSaveSnapshot::default(),
        }
    }
}

#[derive(Debug)]
pub struct AutomationHarness {
    pub(crate) app: App,
}

impl AutomationHarness {
    #[must_use]
    pub fn new(save_root: Option<PathBuf>) -> Self {
        // Headless automation reuses the production shell graph so plans can target the same semantic seams as players.
        Self {
            app: build_headless_app(save_root),
        }
    }

    pub fn step(&mut self) {
        self.app.update();
    }

    /// Boot transition requires two frames: frame 1 runs the `OnEnter(Boot)` schedule
    /// which triggers the state transition to `MainMenu`, frame 2 runs `OnEnter(MainMenu)`
    /// which initializes the shell resources that snapshots depend on.
    pub fn boot_to_main_menu(&mut self) {
        self.step();
        self.step();
    }

    pub fn step_until(
        &mut self,
        predicate: impl Fn(&AutomationSnapshot) -> bool,
        max_frames: u32,
    ) -> AutomationSnapshot {
        for _ in 0..max_frames {
            self.step();
            let snapshot = self.snapshot();
            if predicate(&snapshot) {
                return snapshot;
            }
        }
        panic!(
            "automation harness did not satisfy predicate within {max_frames} frames"
        );
    }

    #[must_use]
    pub fn snapshot(&self) -> AutomationSnapshot {
        capture_snapshot(self.app.world())
    }

    #[must_use]
    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

pub(crate) fn capture_snapshot(world: &World) -> AutomationSnapshot {
    let state = world.resource::<bevy::prelude::State<AppScreenState>>();
    let menu_state = world.resource::<ShellMenuState>();
    let save_state = world.resource::<SaveLoadState>();
    let recovery = world.resource::<RecoveryBannerState>();
    let match_session = world.resource::<MatchSession>();

    AutomationSnapshot {
        screen: screen_from_state(*state.get()),
        menu: AutomationMenuSnapshot {
            panel: panel_from_state(menu_state.panel),
            context: context_from_state(menu_state.context),
            confirmation: menu_state.confirmation.map(confirmation_from_state),
            selected_save: menu_state.selected_save.clone(),
            status_line: menu_state.status_line.clone(),
            shell_status: app_shell_logic::effective_shell_status(
                menu_state,
                save_state,
                recovery,
            ),
            recovery_available: recovery.available,
            recovery_label: recovery.label.clone(),
        },
        match_state: AutomationMatchSnapshot {
            fen: match_session.game_state().to_fen(),
            status: match_session.status(),
            selected_square: match_session.selected_square,
            legal_targets: match_session.legal_targets_for_selected(),
            pending_promotion: match_session.pending_promotion_move,
            last_move: match_session.last_move,
            claimable_draw: match_session.claimable_draw(),
            claimed_draw: match_session.claimed_draw_reason().map(claimed_draw_from_state),
            dirty_recovery: match_session.is_recovery_dirty(),
        },
        saves: AutomationSaveSnapshot {
            manual_saves: save_state.manual_saves.clone(),
            recovery: save_state.recovery.clone(),
            settings: save_state.settings.clone(),
            last_message: save_state.last_message.clone(),
            last_error: save_state.last_error.clone(),
        },
    }
}

fn screen_from_state(state: AppScreenState) -> AutomationScreen {
    match state {
        AppScreenState::Boot => AutomationScreen::Boot,
        AppScreenState::MainMenu => AutomationScreen::MainMenu,
        AppScreenState::MatchLoading => AutomationScreen::MatchLoading,
        AppScreenState::InMatch => AutomationScreen::InMatch,
        AppScreenState::MatchResult => AutomationScreen::MatchResult,
    }
}

fn panel_from_state(panel: MenuPanel) -> AutomationMenuPanel {
    match panel {
        MenuPanel::Home => AutomationMenuPanel::Home,
        MenuPanel::Setup => AutomationMenuPanel::Setup,
        MenuPanel::LoadList => AutomationMenuPanel::LoadList,
        MenuPanel::Settings => AutomationMenuPanel::Settings,
    }
}

fn context_from_state(context: MenuContext) -> AutomationMenuContext {
    match context {
        MenuContext::MainMenu => AutomationMenuContext::MainMenu,
        MenuContext::InMatchOverlay => AutomationMenuContext::InMatchOverlay,
    }
}

fn confirmation_from_state(
    kind: crate::plugins::ConfirmationKind,
) -> AutomationConfirmationKind {
    match kind {
        crate::plugins::ConfirmationKind::AbandonMatch => {
            AutomationConfirmationKind::AbandonMatch
        }
        crate::plugins::ConfirmationKind::DeleteSave => AutomationConfirmationKind::DeleteSave,
        crate::plugins::ConfirmationKind::OverwriteSave => {
            AutomationConfirmationKind::OverwriteSave
        }
    }
}

fn claimed_draw_from_state(kind: ClaimedDrawReason) -> AutomationClaimedDrawReason {
    match kind {
        ClaimedDrawReason::ThreefoldRepetition => {
            AutomationClaimedDrawReason::ThreefoldRepetition
        }
        ClaimedDrawReason::FiftyMoveRule => AutomationClaimedDrawReason::FiftyMoveRule,
    }
}
