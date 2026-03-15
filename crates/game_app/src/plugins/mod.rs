mod app_shell;
mod board_scene;
mod input;
mod move_feedback;
mod piece_view;
mod scaffold;

pub use app_shell::AppShellPlugin;
pub use board_scene::{BoardScenePlugin, BoardSquareVisual};
pub use input::ShellInputPlugin;
pub use move_feedback::MoveFeedbackPlugin;
pub use piece_view::{PieceViewPlugin, PieceVisual};
pub use scaffold::{AiMatchPlugin, ChessAudioPlugin, MenuPlugin, SaveLoadPlugin};
