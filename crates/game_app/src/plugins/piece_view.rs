use bevy::prelude::*;
use chess_core::{Piece, PieceKind, Square};
use std::collections::HashMap;

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
    white_trim_material: Handle<StandardMaterial>,
    black_trim_material: Handle<StandardMaterial>,
    sphere_mesh: Handle<Mesh>,
    cylinder_mesh: Handle<Mesh>,
    cone_mesh: Handle<Mesh>,
    capsule_mesh: Handle<Mesh>,
    ring_mesh: Handle<Mesh>,
    plate_mesh: Handle<Mesh>,
}

impl FromWorld for PieceVisualAssets {
    fn from_world(world: &mut World) -> Self {
        let (piece_white, piece_black, accent) = {
            let theme = world.resource::<ShellTheme>();
            (theme.piece_white, theme.piece_black, theme.accent)
        };

        let (sphere_mesh, cylinder_mesh, cone_mesh, capsule_mesh, ring_mesh, plate_mesh) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(Sphere::new(0.5).mesh().uv(24, 16)),
                meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(28)),
                meshes.add(Cone::new(0.5, 1.0).mesh().resolution(28)),
                meshes.add(Capsule3d::new(0.5, 1.0)),
                meshes.add(Torus::new(0.36, 0.62)),
                meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            )
        };

        let (white_material, black_material, white_trim_material, black_trim_material) = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            (
                materials.add(StandardMaterial {
                    base_color: piece_white,
                    metallic: 0.08,
                    perceptual_roughness: 0.28,
                    reflectance: 0.52,
                    ..default()
                }),
                materials.add(StandardMaterial {
                    base_color: piece_black,
                    metallic: 0.24,
                    perceptual_roughness: 0.25,
                    reflectance: 0.36,
                    ..default()
                }),
                materials.add(StandardMaterial {
                    base_color: piece_white.mix(&accent, 0.22),
                    metallic: 0.32,
                    perceptual_roughness: 0.24,
                    reflectance: 0.56,
                    ..default()
                }),
                materials.add(StandardMaterial {
                    base_color: piece_black.mix(&accent, 0.12),
                    metallic: 0.42,
                    perceptual_roughness: 0.22,
                    reflectance: 0.40,
                    ..default()
                }),
            )
        };

        Self {
            white_material,
            black_material,
            white_trim_material,
            black_trim_material,
            sphere_mesh,
            cylinder_mesh,
            cone_mesh,
            capsule_mesh,
            ring_mesh,
            plate_mesh,
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
        commands.spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            PieceViewRoot,
        ));
    }
}

fn sync_piece_silhouettes_from_match(
    mut commands: Commands,
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
    piece_assets: Res<PieceVisualAssets>,
    root_query: Query<Entity, With<PieceViewRoot>>,
    piece_query: Query<(Entity, &PieceVisual)>,
) {
    let Ok(root_entity) = root_query.single() else {
        return;
    };

    if !match_session.is_changed() && !piece_query.is_empty() {
        return;
    }

    let mut existing_pieces = HashMap::new();
    for (entity, piece_visual) in &piece_query {
        existing_pieces.insert(piece_visual.square, (entity, piece_visual.piece));
    }

    let board_state = match_session.game_state().board();
    let mut to_spawn = Vec::new();
    let mut to_despawn = Vec::new();

    for (square, piece) in board_state.iter() {
        match existing_pieces.remove(square) {
            Some((_entity, existing_piece)) if existing_piece == *piece => {}
            Some((entity, _)) => {
                to_despawn.push(entity);
                to_spawn.push((*square, *piece));
            }
            None => {
                to_spawn.push((*square, *piece));
            }
        }
    }

    for (entity, _) in existing_pieces.into_values() {
        to_despawn.push(entity);
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }

    commands.entity(root_entity).with_children(|parent| {
        for (square, piece) in &to_spawn {
            spawn_piece_visual(parent, &piece_assets, theme.as_ref(), *square, *piece);
        }
    });
}

#[derive(Clone, Copy)]
enum PieceMaterialKind {
    Primary,
    Trim,
}

#[derive(Clone)]
struct PiecePart {
    mesh: Handle<Mesh>,
    material_kind: PieceMaterialKind,
    translation: Vec3,
    scale: Vec3,
    rotation: Quat,
}

impl PiecePart {
    fn new(
        mesh: Handle<Mesh>,
        material_kind: PieceMaterialKind,
        translation: Vec3,
        scale: Vec3,
    ) -> Self {
        Self {
            mesh,
            material_kind,
            translation,
            scale,
            rotation: Quat::IDENTITY,
        }
    }

    fn rotated(
        mesh: Handle<Mesh>,
        material_kind: PieceMaterialKind,
        translation: Vec3,
        scale: Vec3,
        rotation: Quat,
    ) -> Self {
        Self {
            mesh,
            material_kind,
            translation,
            scale,
            rotation,
        }
    }
}

fn spawn_piece_model(
    parent: &mut ChildSpawnerCommands<'_>,
    assets: &PieceVisualAssets,
    piece: Piece,
) {
    let parts = piece_parts(assets, piece.kind);

    for part in parts {
        parent.spawn((
            Mesh3d(part.mesh),
            MeshMaterial3d(piece_material_handle(
                assets,
                piece.side,
                part.material_kind,
            )),
            Transform {
                translation: part.translation,
                rotation: part.rotation,
                scale: part.scale,
            },
        ));
    }
}

fn spawn_piece_visual(
    parent: &mut ChildSpawnerCommands<'_>,
    assets: &PieceVisualAssets,
    theme: &ShellTheme,
    square: Square,
    piece: Piece,
) {
    let piece_translation =
        square_to_board_translation(square, theme.square_size, theme.board_height)
            + Vec3::Y * (piece_height(piece.kind) * 0.5);

    parent
        .spawn((
            PieceVisual { square, piece },
            Transform::from_translation(piece_translation),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|piece_parent| {
            spawn_piece_model(piece_parent, assets, piece);
        });
}

fn piece_material_handle(
    assets: &PieceVisualAssets,
    side: chess_core::Side,
    material_kind: PieceMaterialKind,
) -> Handle<StandardMaterial> {
    match (side, material_kind) {
        (chess_core::Side::White, PieceMaterialKind::Primary) => assets.white_material.clone(),
        (chess_core::Side::Black, PieceMaterialKind::Primary) => assets.black_material.clone(),
        (chess_core::Side::White, PieceMaterialKind::Trim) => assets.white_trim_material.clone(),
        (chess_core::Side::Black, PieceMaterialKind::Trim) => assets.black_trim_material.clone(),
    }
}

fn piece_parts(assets: &PieceVisualAssets, kind: PieceKind) -> Vec<PiecePart> {
    let mut parts = vec![
        PiecePart::new(
            assets.cylinder_mesh.clone(),
            PieceMaterialKind::Primary,
            Vec3::new(0.0, -0.48, 0.0),
            Vec3::new(0.92, 0.16, 0.92),
        ),
        PiecePart::new(
            assets.ring_mesh.clone(),
            PieceMaterialKind::Trim,
            Vec3::new(0.0, -0.37, 0.0),
            Vec3::new(0.88, 0.12, 0.88),
        ),
    ];

    match kind {
        PieceKind::King => {
            parts.extend([
                PiecePart::new(
                    assets.cylinder_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, -0.02, 0.0),
                    Vec3::new(0.52, 0.78, 0.52),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.58, 0.0),
                    Vec3::new(0.46, 0.36, 0.46),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.89, 0.0),
                    Vec3::new(0.14, 0.40, 0.14),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.89, 0.0),
                    Vec3::new(0.40, 0.14, 0.14),
                ),
            ]);
        }
        PieceKind::Queen => {
            parts.extend([
                PiecePart::new(
                    assets.cylinder_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, -0.07, 0.0),
                    Vec3::new(0.48, 0.66, 0.48),
                ),
                PiecePart::new(
                    assets.cone_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.40, 0.0),
                    Vec3::new(0.52, 0.42, 0.52),
                ),
                PiecePart::new(
                    assets.ring_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.76, 0.0),
                    Vec3::new(0.78, 0.14, 0.78),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.94, 0.0),
                    Vec3::new(0.22, 0.22, 0.22),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.32, 0.82, 0.0),
                    Vec3::new(0.12, 0.12, 0.12),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(-0.32, 0.82, 0.0),
                    Vec3::new(0.12, 0.12, 0.12),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.82, 0.32),
                    Vec3::new(0.12, 0.12, 0.12),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.82, -0.32),
                    Vec3::new(0.12, 0.12, 0.12),
                ),
            ]);
        }
        PieceKind::Rook => {
            parts.extend([
                PiecePart::new(
                    assets.cylinder_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, -0.10, 0.0),
                    Vec3::new(0.56, 0.60, 0.56),
                ),
                PiecePart::new(
                    assets.ring_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.28, 0.0),
                    Vec3::new(0.86, 0.14, 0.86),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.46, 0.0),
                    Vec3::new(0.78, 0.16, 0.78),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.28, 0.60, 0.0),
                    Vec3::new(0.14, 0.18, 0.28),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(-0.28, 0.60, 0.0),
                    Vec3::new(0.14, 0.18, 0.28),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.60, 0.28),
                    Vec3::new(0.28, 0.18, 0.14),
                ),
                PiecePart::new(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.60, -0.28),
                    Vec3::new(0.28, 0.18, 0.14),
                ),
            ]);
        }
        PieceKind::Bishop => {
            parts.extend([
                PiecePart::new(
                    assets.capsule_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.00, 0.0),
                    Vec3::new(0.44, 0.70, 0.44),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.62, 0.0),
                    Vec3::new(0.34, 0.34, 0.34),
                ),
                PiecePart::rotated(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.64, 0.0),
                    Vec3::new(0.10, 0.34, 0.42),
                    Quat::from_rotation_z(0.36),
                ),
                PiecePart::new(
                    assets.cone_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.96, 0.0),
                    Vec3::new(0.16, 0.18, 0.16),
                ),
            ]);
        }
        PieceKind::Knight => {
            parts.extend([
                PiecePart::rotated(
                    assets.capsule_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.08, -0.04),
                    Vec3::new(0.40, 0.70, 0.42),
                    Quat::from_rotation_z(-0.42),
                ),
                PiecePart::rotated(
                    assets.plate_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.12, 0.60, 0.10),
                    Vec3::new(0.34, 0.30, 0.46),
                    Quat::from_rotation_z(-0.26) * Quat::from_rotation_x(0.20),
                ),
                PiecePart::new(
                    assets.cone_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.14, 0.86, 0.04),
                    Vec3::new(0.18, 0.22, 0.16),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.20, 0.66, 0.28),
                    Vec3::new(0.08, 0.08, 0.08),
                ),
            ]);
        }
        PieceKind::Pawn => {
            parts.extend([
                PiecePart::new(
                    assets.cylinder_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, -0.12, 0.0),
                    Vec3::new(0.42, 0.44, 0.42),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Primary,
                    Vec3::new(0.0, 0.24, 0.0),
                    Vec3::new(0.34, 0.34, 0.34),
                ),
                PiecePart::new(
                    assets.sphere_mesh.clone(),
                    PieceMaterialKind::Trim,
                    Vec3::new(0.0, 0.64, 0.0),
                    Vec3::new(0.24, 0.24, 0.24),
                ),
            ]);
        }
    }

    parts
}
