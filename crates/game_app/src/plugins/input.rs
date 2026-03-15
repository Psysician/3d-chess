use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use chess_core::{Move, PieceKind};

use crate::app::AppScreenState;
use crate::board_coords::{board_plane_intersection, world_to_square};
use crate::match_state::MatchSession;
use crate::style::ShellTheme;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct HoveredSquare(Option<chess_core::Square>);

// Input resolves to chess squares first and only then to domain actions so legal previews and move execution always flow through chess_core.
pub struct ShellInputPlugin;

impl Plugin for ShellInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredSquare>().add_systems(
            Update,
            (
                pick_square_under_cursor,
                handle_square_clicks,
                handle_keyboard_match_actions,
            )
                .chain()
                .run_if(in_state(AppScreenState::InMatch)),
        );
    }
}

fn pick_square_under_cursor(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    theme: Res<ShellTheme>,
    mut hovered_square: ResMut<HoveredSquare>,
) {
    let Ok(window) = window_query.single() else {
        hovered_square.0 = None;
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        hovered_square.0 = None;
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        hovered_square.0 = None;
        return;
    };
    let Some(intersection) = board_plane_intersection(
        camera,
        camera_transform,
        cursor_position,
        theme.board_height,
    ) else {
        hovered_square.0 = None;
        return;
    };

    hovered_square.0 = world_to_square(intersection, theme.square_size);
}

fn handle_square_clicks(
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    hovered_square: Res<HoveredSquare>,
    mut match_session: ResMut<MatchSession>,
) {
    let Some(mouse_buttons) = mouse_buttons else {
        return;
    };
    if !mouse_buttons.just_pressed(MouseButton::Left) || match_session.status().is_finished() {
        return;
    }

    let Some(clicked_square) = hovered_square.0 else {
        match_session.selected_square = None;
        return;
    };
    if match_session.pending_promotion_move.is_some() {
        return;
    }

    let current_side = match_session.game_state().side_to_move();
    let clicked_piece = match_session.piece_at(clicked_square);

    let Some(selected_square) = match_session.selected_square else {
        if clicked_piece.is_some_and(|piece| piece.side == current_side) {
            match_session.selected_square = Some(clicked_square);
        }
        return;
    };

    if clicked_square == selected_square {
        match_session.clear_interaction();
        return;
    }

    if clicked_piece.is_some_and(|piece| piece.side == current_side) {
        match_session.selected_square = Some(clicked_square);
        return;
    }

    let candidate_moves: Vec<_> = match_session
        .game_state()
        .legal_moves()
        .into_iter()
        .filter(|candidate| candidate.from() == selected_square && candidate.to() == clicked_square)
        .collect();

    if candidate_moves.is_empty() {
        match_session.selected_square = None;
        return;
    }

    if candidate_moves.iter().any(|candidate| candidate.promotion().is_some()) {
        match_session.pending_promotion_move = Some(Move::new(selected_square, clicked_square));
        return;
    }

    let _ = match_session.apply_move(candidate_moves[0]);
}

fn handle_keyboard_match_actions(
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    mut match_session: ResMut<MatchSession>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    let Some(keyboard_input) = keyboard_input else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        if match_session.pending_promotion_move.is_some() || match_session.selected_square.is_some() {
            match_session.clear_interaction();
        } else {
            next_state.set(AppScreenState::MainMenu);
        }
        return;
    }

    let Some(pending_move) = match_session.pending_promotion_move else {
        return;
    };

    let promotion_kind = if keyboard_input.just_pressed(KeyCode::KeyQ) {
        Some(PieceKind::Queen)
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        Some(PieceKind::Rook)
    } else if keyboard_input.just_pressed(KeyCode::KeyB) {
        Some(PieceKind::Bishop)
    } else if keyboard_input.just_pressed(KeyCode::KeyN) {
        Some(PieceKind::Knight)
    } else {
        None
    };

    if let Some(promotion_kind) = promotion_kind {
        let _ = match_session.apply_move(Move::with_promotion(
            pending_move.from(),
            pending_move.to(),
            promotion_kind,
        ));
    }
}
