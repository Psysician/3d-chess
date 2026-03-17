// Shared match interaction helpers keep raw input and automation on one
// legality path while `MatchSession` stays authoritative.
// (refs: DL-003, DL-006)

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use chess_core::{Move, PieceKind, Square};

use super::menu::{MenuAction, MenuContext, ShellMenuState};
use super::save_load::SaveLoadRequest;
use crate::app::AppScreenState;
use crate::automation::{AutomationError, AutomationMatchAction, AutomationResult};
use crate::board_coords::{board_plane_intersection, world_to_square};
use crate::match_state::MatchSession;
use crate::style::ShellTheme;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct HoveredSquare(Option<chess_core::Square>);

// Input resolves to chess squares first and only then to shell events so recovery snapshots mirror domain intent.
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
    menu_state: Res<ShellMenuState>,
    mut match_session: ResMut<MatchSession>,
) {
    let Some(mouse_buttons) = mouse_buttons else {
        return;
    };
    if overlay_captures_match_input(&menu_state)
        || !mouse_buttons.just_pressed(MouseButton::Left)
        || match_session.status().is_finished()
    {
        return;
    }

    apply_square_interaction(match_session.as_mut(), hovered_square.0);
}

fn handle_keyboard_match_actions(
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    menu_state: Res<ShellMenuState>,
    mut match_session: ResMut<MatchSession>,
    mut menu_actions: MessageWriter<MenuAction>,
    mut save_requests: MessageWriter<SaveLoadRequest>,
) {
    let Some(keyboard_input) = keyboard_input else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        if menu_state.confirmation.is_some() {
            menu_actions.write(MenuAction::CancelModal);
        } else if overlay_captures_match_input(&menu_state) {
            menu_actions.write(MenuAction::ResumeMatch);
        } else if match_session.pending_promotion_move.is_some()
            || match_session.selected_square.is_some()
        {
            clear_match_interaction(match_session.as_mut());
        } else {
            menu_actions.write(MenuAction::PauseMatch);
        }
        return;
    }

    if overlay_captures_match_input(&menu_state) {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::F5) && match_session.pending_promotion_move.is_none() {
        save_requests.write(SaveLoadRequest::SaveManual {
            label: String::from("Quick Save"),
            slot_id: None,
        });
    }

    let Some(_pending_move) = match_session.pending_promotion_move else {
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
        let _ = apply_promotion_choice(match_session.as_mut(), promotion_kind);
    }
}

pub(crate) fn apply_match_action(
    match_session: &mut MatchSession,
    action: &AutomationMatchAction,
) -> AutomationResult<()> {
    match action {
        AutomationMatchAction::SelectSquare { square } => {
            apply_square_interaction(match_session, Some(*square));
            Ok(())
        }
        AutomationMatchAction::SubmitMove {
            from,
            to,
            promotion,
        } => {
            apply_square_interaction(match_session, Some(*from));
            apply_square_interaction(match_session, Some(*to));
            if let Some(piece) = promotion {
                apply_promotion_choice(match_session, *piece)?;
            }
            Ok(())
        }
        AutomationMatchAction::ChoosePromotion { piece } => {
            apply_promotion_choice(match_session, *piece)
        }
        AutomationMatchAction::ClearInteraction => {
            clear_match_interaction(match_session);
            Ok(())
        }
    }
}

pub(crate) fn apply_square_interaction(
    match_session: &mut MatchSession,
    clicked_square: Option<Square>,
) {
    let Some(clicked_square) = clicked_square else {
        clear_match_interaction(match_session);
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
            match_session.mark_recovery_dirty();
        }
        return;
    };

    if clicked_square == selected_square {
        clear_match_interaction(match_session);
        return;
    }

    if clicked_piece.is_some_and(|piece| piece.side == current_side) {
        match_session.selected_square = Some(clicked_square);
        match_session.mark_recovery_dirty();
        return;
    }

    let candidate_moves: Vec<_> = match_session
        .game_state()
        .legal_moves()
        .into_iter()
        .filter(|candidate| {
            candidate.from() == selected_square && candidate.to() == clicked_square
        })
        .collect();

    if candidate_moves.is_empty() {
        clear_match_interaction(match_session);
        return;
    }

    if candidate_moves.iter().any(|candidate| candidate.promotion().is_some()) {
        match_session.pending_promotion_move = Some(Move::new(selected_square, clicked_square));
        match_session.mark_recovery_dirty();
        return;
    }

    let _ = match_session.apply_move(candidate_moves[0]);
}

pub(crate) fn clear_match_interaction(match_session: &mut MatchSession) {
    match_session.clear_interaction();
    match_session.mark_recovery_dirty();
}

pub(crate) fn apply_promotion_choice(
    match_session: &mut MatchSession,
    promotion_kind: PieceKind,
) -> AutomationResult<()> {
    let Some(pending_move) = match_session.pending_promotion_move else {
        return Err(AutomationError::PromotionUnavailable);
    };
    let _ = match_session.apply_move(Move::with_promotion(
        pending_move.from(),
        pending_move.to(),
        promotion_kind,
    ));
    Ok(())
}

fn overlay_captures_match_input(menu_state: &ShellMenuState) -> bool {
    menu_state.context == MenuContext::InMatchOverlay
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::system::SystemState;
    use chess_core::Square;

    type KeyboardActionSystemState<'w, 's> = SystemState<(
        Option<Res<'w, ButtonInput<KeyCode>>>,
        Res<'w, ShellMenuState>,
        ResMut<'w, MatchSession>,
        MessageWriter<'w, MenuAction>,
        MessageWriter<'w, SaveLoadRequest>,
    )>;

    type SquareClickSystemState<'w, 's> = SystemState<(
        Option<Res<'w, ButtonInput<MouseButton>>>,
        Res<'w, HoveredSquare>,
        Res<'w, ShellMenuState>,
        ResMut<'w, MatchSession>,
    )>;

    #[test]
    fn in_match_overlay_blocks_board_clicks() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(None));
        world.insert_resource(ShellMenuState {
            context: MenuContext::InMatchOverlay,
            ..Default::default()
        });

        let mut match_session = MatchSession::start_local_match();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        world.insert_resource(match_session);

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        let (mouse_buttons, hovered_square, menu_state, match_session) =
            system_state.get_mut(&mut world);
        handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);

        assert_eq!(
            world.resource::<MatchSession>().selected_square,
            Some(Square::from_algebraic("e2").expect("valid square"))
        );
    }

    #[test]
    fn overlay_helper_tracks_pause_context() {
        assert!(overlay_captures_match_input(&ShellMenuState {
            context: MenuContext::InMatchOverlay,
            ..Default::default()
        }));
        assert!(!overlay_captures_match_input(&ShellMenuState::default()));
    }

    #[test]
    fn clicking_without_hover_clears_selection_and_marks_recovery_dirty() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(None));
        world.insert_resource(ShellMenuState::default());

        let mut match_session = MatchSession::start_local_match();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        match_session.mark_recovery_persisted();
        world.insert_resource(match_session);

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        let (mouse_buttons, hovered_square, menu_state, match_session) =
            system_state.get_mut(&mut world);
        handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);

        let match_session = world.resource::<MatchSession>();
        assert_eq!(match_session.selected_square, None);
        assert!(match_session.is_recovery_dirty());
    }

    #[test]
    fn clicking_selected_square_deselects_and_marks_recovery_dirty() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(Some(
            Square::from_algebraic("e2").expect("valid square"),
        )));
        world.insert_resource(ShellMenuState::default());

        let mut match_session = MatchSession::start_local_match();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        match_session.mark_recovery_persisted();
        world.insert_resource(match_session);

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        let (mouse_buttons, hovered_square, menu_state, match_session) =
            system_state.get_mut(&mut world);
        handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);

        let match_session = world.resource::<MatchSession>();
        assert_eq!(match_session.selected_square, None);
        assert!(match_session.is_recovery_dirty());
    }

    #[test]
    fn clicking_friendly_piece_selects_and_reselects_current_side_piece() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(Some(
            Square::from_algebraic("e2").expect("valid square"),
        )));
        world.insert_resource(ShellMenuState::default());
        world.insert_resource(MatchSession::start_local_match());

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        {
            let (mouse_buttons, hovered_square, menu_state, match_session) =
                system_state.get_mut(&mut world);
            handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);
        }
        assert_eq!(
            world.resource::<MatchSession>().selected_square,
            Some(Square::from_algebraic("e2").expect("valid square"))
        );

        world.insert_resource(HoveredSquare(Some(
            Square::from_algebraic("d2").expect("valid square"),
        )));
        {
            let (mouse_buttons, hovered_square, menu_state, match_session) =
                system_state.get_mut(&mut world);
            handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);
        }
        assert_eq!(
            world.resource::<MatchSession>().selected_square,
            Some(Square::from_algebraic("d2").expect("valid square"))
        );
    }

    #[test]
    fn clicking_illegal_target_clears_selection() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(Some(
            Square::from_algebraic("e5").expect("valid square"),
        )));
        world.insert_resource(ShellMenuState::default());

        let mut match_session = MatchSession::start_local_match();
        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
        world.insert_resource(match_session);

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        let (mouse_buttons, hovered_square, menu_state, match_session) =
            system_state.get_mut(&mut world);
        handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);

        assert_eq!(world.resource::<MatchSession>().selected_square, None);
    }

    #[test]
    fn promotion_target_stages_pending_promotion_move() {
        let mut world = World::new();
        let mut mouse_buttons = ButtonInput::<MouseButton>::default();
        mouse_buttons.press(MouseButton::Left);
        world.insert_resource(mouse_buttons);
        world.insert_resource(HoveredSquare(Some(
            Square::from_algebraic("e8").expect("valid square"),
        )));
        world.insert_resource(ShellMenuState::default());

        let mut match_session = MatchSession::start_local_match();
        match_session.replace_game_state(
            chess_core::GameState::from_fen("7k/4P3/8/8/8/8/8/4K3 w - - 0 1")
                .expect("fixture FEN should parse"),
        );
        match_session.selected_square = Some(Square::from_algebraic("e7").expect("valid square"));
        world.insert_resource(match_session);

        let mut system_state: SquareClickSystemState<'_, '_> = SystemState::new(&mut world);
        let (mouse_buttons, hovered_square, menu_state, match_session) =
            system_state.get_mut(&mut world);
        handle_square_clicks(mouse_buttons, hovered_square, menu_state, match_session);

        assert_eq!(
            world.resource::<MatchSession>().pending_promotion_move,
            Some(Move::new(
                Square::from_algebraic("e7").expect("valid square"),
                Square::from_algebraic("e8").expect("valid square"),
            ))
        );
    }

    #[test]
    fn escape_clears_pending_promotion_before_pause_overlay() {
        let mut app = App::new();
        app.add_message::<MenuAction>();
        app.add_message::<SaveLoadRequest>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ShellMenuState::default());

        let mut match_session = MatchSession::start_local_match();
        match_session.pending_promotion_move = Some(Move::new(
            Square::from_algebraic("e7").expect("valid square"),
            Square::from_algebraic("e8").expect("valid square"),
        ));
        app.insert_resource(match_session);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        let mut system_state: KeyboardActionSystemState<'_, '_> = SystemState::new(app.world_mut());
        let (keyboard_input, menu_state, match_session, menu_actions, save_requests) =
            system_state.get_mut(app.world_mut());
        handle_keyboard_match_actions(
            keyboard_input,
            menu_state,
            match_session,
            menu_actions,
            save_requests,
        );

        let match_session = app.world().resource::<MatchSession>();
        assert_eq!(match_session.pending_promotion_move, None);
        assert!(match_session.is_recovery_dirty());
    }
}
