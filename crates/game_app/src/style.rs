use bevy::prelude::*;

#[derive(Resource)]
pub struct ShellTheme {
    pub clear_color: Color,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
    pub board_light: Color,
    pub board_dark: Color,
    pub plinth_color: Color,
    pub piece_white: Color,
    pub piece_black: Color,
    pub accent: Color,
    pub ui_text: Color,
    pub ui_panel: Color,
    pub camera_focus: Vec3,
    pub camera_radius: f32,
    pub camera_height: f32,
    pub orbit_speed: f32,
    pub square_size: f32,
    pub board_height: f32,
}

impl Default for ShellTheme {
    fn default() -> Self {
        Self {
            clear_color: Color::srgb(0.035, 0.043, 0.065),
            ambient_color: Color::srgb(0.70, 0.74, 0.80),
            ambient_brightness: 180.0,
            board_light: Color::srgb(0.79, 0.70, 0.55),
            board_dark: Color::srgb(0.18, 0.20, 0.25),
            plinth_color: Color::srgb(0.11, 0.12, 0.16),
            piece_white: Color::srgb(0.90, 0.88, 0.83),
            piece_black: Color::srgb(0.20, 0.22, 0.28),
            accent: Color::srgb(0.91, 0.58, 0.31),
            ui_text: Color::srgb(0.94, 0.95, 0.97),
            ui_panel: Color::srgba(0.06, 0.08, 0.12, 0.76),
            camera_focus: Vec3::new(0.0, 0.45, 0.0),
            camera_radius: 12.0,
            camera_height: 8.2,
            orbit_speed: 0.16,
            square_size: 1.05,
            board_height: 0.16,
        }
    }
}
