pub mod board_coords;

mod app;
mod match_state;
mod plugins;
mod style;

pub use app::{APP_TITLE, AppScreenState, build_app, run};
pub use match_state::{
    ClaimedDrawReason, MatchLaunchIntent, MatchSession, MatchSessionSummary, PendingLoadedSnapshot,
};
pub use plugins::{
    AiMatchPlugin, AppShellPlugin, BoardScenePlugin, BoardSquareVisual, ChessAudioPlugin,
    ConfirmationKind, MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin, PieceViewPlugin,
    PieceVisual, RecoveryBannerState, SaveLoadPlugin, SaveLoadRequest, SaveLoadState,
    SaveRootOverride, SessionStoreResource, ShellInputPlugin, ShellMenuState,
};
pub use style::ShellTheme;
