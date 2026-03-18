use bevy::prelude::*;
use chess_core::{GameStatus, Square};

use crate::app::AppScreenState;
use crate::board_coords::square_to_board_translation;
use crate::match_state::MatchSession;
use crate::style::ShellTheme;

pub struct BoardScenePlugin;

impl Plugin for BoardScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreenState::MainMenu), spawn_board_scene)
            .add_systems(Update, update_square_visual_state);
    }
}

#[derive(Component)]
struct BoardSceneRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardSquareVisual {
    pub square: Square,
    is_light_square: bool,
}

fn spawn_board_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Res<ShellTheme>,
    existing_root: Query<Entity, With<BoardSceneRoot>>,
) {
    if !existing_root.is_empty() {
        return;
    }

    let plinth_mesh = meshes.add(Cuboid::new(10.8, 0.55, 10.8));
    let plinth_material = materials.add(StandardMaterial {
        base_color: theme.plinth_color,
        metallic: 0.08,
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    });
    let accent_strip_material = materials.add(StandardMaterial {
        base_color: theme.accent,
        metallic: 0.30,
        perceptual_roughness: 0.42,
        reflectance: 0.46,
        ..default()
    });
    let square_mesh = meshes.add(Cuboid::new(
        theme.square_size,
        theme.board_height,
        theme.square_size,
    ));
    let accent_mesh = meshes.add(Cuboid::new(8.7, 0.04, 0.28));

    commands
        .spawn((Transform::default(), BoardSceneRoot))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(plinth_mesh.clone()),
                MeshMaterial3d(plinth_material),
                Transform::from_xyz(0.0, -0.22, 0.0),
            ));

            parent.spawn((
                Mesh3d(accent_mesh.clone()),
                MeshMaterial3d(accent_strip_material.clone()),
                Transform::from_xyz(0.0, 0.14, 4.45),
            ));
            parent.spawn((
                Mesh3d(accent_mesh),
                MeshMaterial3d(accent_strip_material),
                Transform::from_xyz(0.0, 0.14, -4.45),
            ));

            for file in 0_u8..8 {
                for rank in 0_u8..8 {
                    let square = Square::from_coords_unchecked(file, rank);
                    let square_translation =
                        square_to_board_translation(square, theme.square_size, 0.0);
                    let is_light_square = (usize::from(file) + usize::from(rank)) % 2 == 0;
                    let material = materials.add(StandardMaterial {
                        base_color: base_square_color(theme.as_ref(), is_light_square),
                        metallic: if is_light_square { 0.02 } else { 0.05 },
                        perceptual_roughness: if is_light_square { 0.62 } else { 0.58 },
                        reflectance: if is_light_square { 0.28 } else { 0.22 },
                        ..default()
                    });

                    parent.spawn((
                        BoardSquareVisual {
                            square,
                            is_light_square,
                        },
                        Mesh3d(square_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(square_translation),
                    ));
                }
            }
        });
}

fn update_square_visual_state(
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    square_query: Query<(&BoardSquareVisual, &MeshMaterial3d<StandardMaterial>)>,
) {
    if !match_session.is_changed() && !theme.is_changed() {
        return;
    }

    let legal_targets = match_session.legal_targets_for_selected();
    let checked_king_square = match match_session.status() {
        GameStatus::Ongoing { in_check: true, .. } => match_session
            .game_state()
            .board()
            .king_square(match_session.game_state().side_to_move()),
        _ => None,
    };

    for (square_visual, material_handle) in &square_query {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        material.base_color = if checked_king_square == Some(square_visual.square) {
            Color::srgb(0.73, 0.24, 0.20)
        } else if legal_targets.contains(&square_visual.square) {
            Color::srgb(0.32, 0.48, 0.64)
        } else if match_session.selected_square == Some(square_visual.square) {
            theme.accent
        } else {
            base_square_color(theme.as_ref(), square_visual.is_light_square)
        };
    }
}

fn base_square_color(theme: &ShellTheme, is_light_square: bool) -> Color {
    if is_light_square {
        theme.board_light
    } else {
        theme.board_dark
    }
}
