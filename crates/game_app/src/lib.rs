// Automation re-exports stay separate from `run` so harness consumers can opt
// into the contract without changing the GUI entry point.
// (refs: DL-004, DL-005)

pub mod board_coords;
mod automation;

#[cfg(feature = "automation-transport")]
pub mod automation_transport;

mod app;
mod match_state;
mod plugins;
mod style;

pub use app::{APP_TITLE, AppScreenState, build_app, build_headless_app, run};
pub use automation::{
    AutomationClaimedDrawReason, AutomationCommand, AutomationConfirmationKind,
    AutomationError, AutomationHarness, AutomationMatchAction, AutomationMatchSnapshot,
    AutomationMenuContext, AutomationMenuPanel, AutomationMenuSnapshot,
    AutomationNavigationAction, AutomationResult, AutomationSaveAction,
    AutomationSaveSnapshot, AutomationScreen, AutomationSettingsAction,
    AutomationSnapshot,
};
pub use match_state::{
    ClaimedDrawReason, MatchLaunchIntent, MatchSession, MatchSessionSummary, PendingLoadedSnapshot,
};
pub use plugins::{
    AiMatchPlugin, AppShellPlugin, AutomationPlugin, BoardScenePlugin, BoardSquareVisual,
    ChessAudioPlugin, ConfirmationKind, MenuAction, MenuContext, MenuPanel, MenuPlugin,
    MoveFeedbackPlugin, PieceViewPlugin, PieceVisual, RecoveryBannerState, SaveLoadPlugin,
    SaveLoadRequest, SaveLoadState, SaveRootOverride, SessionStoreResource, ShellInputPlugin,
    ShellMenuState,
};
pub use style::ShellTheme;

#[doc(hidden)]
pub mod test_support {
    pub use crate::plugins::app_shell_logic;
    pub use crate::plugins::save_load_logic;
}
