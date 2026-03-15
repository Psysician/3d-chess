use bevy::prelude::*;

pub struct MenuPlugin;
pub struct SaveLoadPlugin;
pub struct AiMatchPlugin;
pub struct ChessAudioPlugin;

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
