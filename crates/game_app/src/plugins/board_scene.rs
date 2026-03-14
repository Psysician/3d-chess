use bevy::prelude::*;

use crate::app::AppScreenState;
use crate::style::ShellTheme;

pub struct BoardScenePlugin;

impl Plugin for BoardScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreenState::MainMenu), spawn_board_scene);
    }
}

#[derive(Component)]
struct BoardSceneRoot;

fn spawn_board_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Res<ShellTheme>,
) {
    let plinth_mesh = meshes.add(Cuboid::new(10.8, 0.55, 10.8));
    let plinth_material = materials.add(StandardMaterial {
        base_color: theme.plinth_color,
        metallic: 0.08,
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    });
    let light_square_material = materials.add(StandardMaterial {
        base_color: theme.board_light,
        metallic: 0.02,
        perceptual_roughness: 0.62,
        reflectance: 0.28,
        ..default()
    });
    let dark_square_material = materials.add(StandardMaterial {
        base_color: theme.board_dark,
        metallic: 0.05,
        perceptual_roughness: 0.58,
        reflectance: 0.22,
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
                    let x = board_axis(file, theme.square_size);
                    let z = board_axis(rank, theme.square_size);
                    let is_light_square = (usize::from(file) + usize::from(rank)) % 2 == 0;
                    let material = if is_light_square {
                        light_square_material.clone()
                    } else {
                        dark_square_material.clone()
                    };

                    parent.spawn((
                        Mesh3d(square_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_xyz(x, 0.0, z),
                    ));
                }
            }
        });
}

fn board_axis(index: u8, square_size: f32) -> f32 {
    (f32::from(index) - 3.5) * square_size
}
