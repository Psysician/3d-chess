use bevy::prelude::*;
use chess_core::{Piece, PieceKind, Square};

use crate::app::AppScreenState;
use crate::board_coords::square_to_board_translation;
use crate::match_state::MatchSession;
use crate::style::ShellTheme;

pub struct PieceViewPlugin;

impl Plugin for PieceViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceVisualAssets>()
            .add_systems(OnEnter(AppScreenState::MainMenu), ensure_piece_view_root)
            .add_systems(Update, sync_piece_silhouettes_from_match);
    }
}

#[derive(Component)]
struct PieceViewRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceVisual {
    pub square: Square,
    pub piece: Piece,
}

#[derive(Resource)]
struct PieceVisualAssets {
    white_material: Handle<StandardMaterial>,
    black_material: Handle<StandardMaterial>,
    king_mesh: Handle<Mesh>,
    queen_mesh: Handle<Mesh>,
    rook_mesh: Handle<Mesh>,
    bishop_mesh: Handle<Mesh>,
    knight_mesh: Handle<Mesh>,
    pawn_mesh: Handle<Mesh>,
}

impl FromWorld for PieceVisualAssets {
    fn from_world(world: &mut World) -> Self {
        let (piece_white, piece_black, footprint) = {
            let theme = world.resource::<ShellTheme>();
            (
                theme.piece_white,
                theme.piece_black,
                theme.square_size * 0.52,
            )
        };

        let (
            king_mesh,
            queen_mesh,
            rook_mesh,
            bishop_mesh,
            knight_mesh,
            pawn_mesh,
        ) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::King), footprint)),
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::Queen), footprint)),
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::Rook), footprint)),
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::Bishop), footprint)),
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::Knight), footprint)),
                meshes.add(Cuboid::new(footprint, piece_height(PieceKind::Pawn), footprint)),
            )
        };

        let (white_material, black_material) = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            (
                materials.add(StandardMaterial {
                    base_color: piece_white,
                    metallic: 0.16,
                    perceptual_roughness: 0.36,
                    reflectance: 0.48,
                    ..default()
                }),
                materials.add(StandardMaterial {
                    base_color: piece_black,
                    metallic: 0.22,
                    perceptual_roughness: 0.32,
                    reflectance: 0.34,
                    ..default()
                }),
            )
        };

        Self {
            white_material,
            black_material,
            king_mesh,
            queen_mesh,
            rook_mesh,
            bishop_mesh,
            knight_mesh,
            pawn_mesh,
        }
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

fn ensure_piece_view_root(
    mut commands: Commands,
    existing_root: Query<Entity, With<PieceViewRoot>>,
) {
    if existing_root.is_empty() {
        commands.spawn((Transform::default(), PieceViewRoot));
    }
}

fn sync_piece_silhouettes_from_match(
    mut commands: Commands,
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
    piece_assets: Res<PieceVisualAssets>,
    root_query: Query<Entity, With<PieceViewRoot>>,
    piece_query: Query<Entity, With<PieceVisual>>,
) {
    let Ok(root_entity) = root_query.single() else {
        return;
    };

    if !match_session.is_changed() && !piece_query.is_empty() {
        return;
    }

    // M2 replaces the shell-only starting layout with GameState-driven piece sync so the visual board cannot drift from chess_core.
    for entity in &piece_query {
        commands.entity(entity).despawn();
    }

    let board_state = match_session.game_state().board();
    commands.entity(root_entity).with_children(|parent| {
        for (square, piece) in board_state.iter() {
            let piece_translation =
                square_to_board_translation(*square, theme.square_size, theme.board_height)
                    + Vec3::Y * (piece_height(piece.kind) * 0.5);
            let material = if piece.side == chess_core::Side::White {
                piece_assets.white_material.clone()
            } else {
                piece_assets.black_material.clone()
            };

            parent.spawn((
                PieceVisual {
                    square: *square,
                    piece: *piece,
                },
                Mesh3d(piece_mesh(&piece_assets, piece.kind)),
                MeshMaterial3d(material),
                Transform::from_translation(piece_translation),
            ));
        }
    });
}

fn piece_mesh(piece_assets: &PieceVisualAssets, kind: PieceKind) -> Handle<Mesh> {
    match kind {
        PieceKind::King => piece_assets.king_mesh.clone(),
        PieceKind::Queen => piece_assets.queen_mesh.clone(),
        PieceKind::Rook => piece_assets.rook_mesh.clone(),
        PieceKind::Bishop => piece_assets.bishop_mesh.clone(),
        PieceKind::Knight => piece_assets.knight_mesh.clone(),
        PieceKind::Pawn => piece_assets.pawn_mesh.clone(),
    }
}
