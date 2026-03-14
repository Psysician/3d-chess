use bevy::prelude::*;
use chess_core::{PieceKind, Side};

use crate::app::AppScreenState;
use crate::style::ShellTheme;

pub struct PieceViewPlugin;

impl Plugin for PieceViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreenState::MainMenu), spawn_piece_silhouettes);
    }
}

#[derive(Component)]
struct PieceViewRoot;

fn spawn_piece_silhouettes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Res<ShellTheme>,
) {
    let white_material = materials.add(StandardMaterial {
        base_color: theme.piece_white,
        metallic: 0.16,
        perceptual_roughness: 0.36,
        reflectance: 0.48,
        ..default()
    });
    let black_material = materials.add(StandardMaterial {
        base_color: theme.piece_black,
        metallic: 0.22,
        perceptual_roughness: 0.32,
        reflectance: 0.34,
        ..default()
    });

    commands
        .spawn((Transform::default(), PieceViewRoot))
        .with_children(|parent| {
            for (side, home_rank, pawn_rank) in
                [(Side::White, 0_u8, 1_u8), (Side::Black, 7_u8, 6_u8)]
            {
                for file in 0_u8..8 {
                    let kind = back_rank_piece(file);
                    let back_rank_height = piece_height(kind);
                    let pawn_height = piece_height(PieceKind::Pawn);

                    parent.spawn(piece_bundle(
                        &mut meshes,
                        if side == Side::White {
                            white_material.clone()
                        } else {
                            black_material.clone()
                        },
                        file,
                        home_rank,
                        back_rank_height,
                        theme.square_size,
                        theme.board_height,
                    ));
                    parent.spawn(piece_bundle(
                        &mut meshes,
                        if side == Side::White {
                            white_material.clone()
                        } else {
                            black_material.clone()
                        },
                        file,
                        pawn_rank,
                        pawn_height,
                        theme.square_size,
                        theme.board_height,
                    ));
                }
            }
        });
}

fn piece_bundle(
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    file: u8,
    rank: u8,
    height: f32,
    square_size: f32,
    board_height: f32,
) -> impl Bundle {
    (
        Mesh3d(meshes.add(Cuboid::new(square_size * 0.52, height, square_size * 0.52))),
        MeshMaterial3d(material),
        Transform::from_xyz(
            board_axis(file, square_size),
            board_height * 0.5 + height * 0.5,
            board_axis(rank, square_size),
        ),
    )
}

fn back_rank_piece(file: u8) -> PieceKind {
    match file {
        0 | 7 => PieceKind::Rook,
        1 | 6 => PieceKind::Knight,
        2 | 5 => PieceKind::Bishop,
        3 => PieceKind::Queen,
        4 => PieceKind::King,
        _ => PieceKind::Pawn,
    }
}

fn piece_height(kind: PieceKind) -> f32 {
    match kind {
        PieceKind::King => 1.55,
        PieceKind::Queen => 1.42,
        PieceKind::Rook => 1.12,
        PieceKind::Bishop => 1.28,
        PieceKind::Knight => 1.20,
        PieceKind::Pawn => 0.86,
    }
}

fn board_axis(index: u8, square_size: f32) -> f32 {
    (f32::from(index) - 3.5) * square_size
}
