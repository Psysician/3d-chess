use bevy::prelude::*;

pub struct ShellInputPlugin;
pub struct MoveFeedbackPlugin;
pub struct MenuPlugin;
pub struct SaveLoadPlugin;
pub struct AiMatchPlugin;
pub struct ChessAudioPlugin;

impl Plugin for ShellInputPlugin {
    fn build(&self, _app: &mut App) {}
}

impl Plugin for MoveFeedbackPlugin {
    fn build(&self, _app: &mut App) {}
}

impl Plugin for MenuPlugin {
    fn build(&self, _app: &mut App) {}
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, _app: &mut App) {}
}

impl Plugin for AiMatchPlugin {
    fn build(&self, _app: &mut App) {}
}

impl Plugin for ChessAudioPlugin {
    fn build(&self, _app: &mut App) {}
}
