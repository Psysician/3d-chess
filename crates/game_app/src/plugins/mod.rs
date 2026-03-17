// Automation plugin exports remain opt-in so headless harness composition does
// not alter the default player plugin graph. (refs: DL-004, DL-005)

mod automation;
mod app_shell;
pub mod app_shell_logic;
mod board_scene;
mod input;
mod menu;
mod move_feedback;
mod piece_view;
mod save_load;
pub mod save_load_logic;
mod scaffold;

pub use automation::AutomationPlugin;
pub use app_shell::AppShellPlugin;
pub use board_scene::{BoardScenePlugin, BoardSquareVisual};
pub use input::ShellInputPlugin;
pub use menu::{
    ConfirmationKind, MenuAction, MenuContext, MenuPanel, MenuPlugin, RecoveryBannerState,
    ShellMenuState,
};
pub use move_feedback::MoveFeedbackPlugin;
pub use piece_view::{PieceViewPlugin, PieceVisual};
pub use save_load::{
    SaveLoadPlugin, SaveLoadRequest, SaveLoadState, SaveRootOverride, SessionStoreResource,
};
pub use scaffold::{AiMatchPlugin, ChessAudioPlugin};
