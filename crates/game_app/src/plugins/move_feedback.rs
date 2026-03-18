use bevy::prelude::*;
use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Side, WinReason};

use super::piece_view::PieceVisual;
use super::save_load::SaveLoadState;
use crate::app::AppScreenState;
use crate::match_state::{ClaimedDrawReason, MatchSession};
use crate::style::ShellTheme;

pub struct MoveFeedbackPlugin;

impl Plugin for MoveFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppScreenState::InMatch), spawn_match_hud)
            .add_systems(OnExit(AppScreenState::InMatch), cleanup_match_hud)
            .add_systems(
                Update,
                sync_match_hud.run_if(in_state(AppScreenState::InMatch)),
            )
            .add_systems(
                Update,
                animate_active_move.run_if(in_state(AppScreenState::InMatch)),
            )
            .add_systems(
                Update,
                update_claim_draw_banner.run_if(in_state(AppScreenState::InMatch)),
            )
            .add_systems(
                Update,
                handle_claim_draw_button_actions.run_if(in_state(AppScreenState::InMatch)),
            );
    }
}

#[derive(Component)]
struct MatchHudRoot;

#[derive(Component)]
struct TurnStatusText;

#[derive(Component)]
struct MatchStatusText;

#[derive(Component)]
struct PromotionHintText;

#[derive(Component)]
struct PersistenceStatusText;

#[derive(Component)]
struct ClaimDrawButton;

type HudTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        Option<&'static TurnStatusText>,
        Option<&'static MatchStatusText>,
        Option<&'static PromotionHintText>,
        Option<&'static PersistenceStatusText>,
    ),
>;

fn spawn_match_hud(mut commands: Commands, theme: Res<ShellTheme>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(24.0),
                left: Val::Px(24.0),
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(theme.ui_panel),
            MatchHudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("White to move"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(theme.ui_text),
                TurnStatusText,
            ));
            parent.spawn((
                Text::new("Select a piece to begin."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(theme.accent),
                MatchStatusText,
            ));
            parent.spawn((
                Text::new("Promotion uses Q / R / B / N."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.ui_text),
                PromotionHintText,
            ));
            parent.spawn((
                Text::new("Interrupted-session recovery is waiting for the next autosave."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.ui_text),
                PersistenceStatusText,
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(theme.accent),
                    Visibility::Hidden,
                    ClaimDrawButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Claim Draw"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
                });
        });
}

fn cleanup_match_hud(mut commands: Commands, hud_query: Query<Entity, With<MatchHudRoot>>) {
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }
}

fn sync_match_hud(
    match_session: Res<MatchSession>,
    save_state: Res<SaveLoadState>,
    mut text_query: HudTextQuery<'_, '_>,
) {
    if !match_session.is_changed() && !save_state.is_changed() {
        return;
    }

    let turn_label = format!(
        "{} to move",
        side_label(match_session.game_state().side_to_move())
    );
    let status_label = match_status_label(&match_session);
    let promotion_hint_label = if let Some(pending_move) = match_session.pending_promotion_move {
        format!(
            "Promotion pending for {} -> {}. Choose Q / R / B / N or use the overlay.",
            pending_move.from(),
            pending_move.to()
        )
    } else if let Some(last_move) = match_session.last_move {
        format!("Last move: {last_move}")
    } else {
        String::from("Promotion uses Q / R / B / N.")
    };
    let persistence_label = save_state
        .last_error
        .clone()
        .or_else(|| save_state.last_message.clone())
        .unwrap_or_else(|| {
            if match_session.is_recovery_dirty() {
                String::from("Interrupted-session recovery is waiting for the next autosave.")
            } else {
                String::from("Interrupted-session recovery is current.")
            }
        });

    for (mut text, turn_marker, status_marker, promotion_marker, persistence_marker) in
        &mut text_query
    {
        if turn_marker.is_some() {
            text.0 = turn_label.clone();
        } else if status_marker.is_some() {
            text.0 = status_label.clone();
        } else if promotion_marker.is_some() {
            text.0 = promotion_hint_label.clone();
        } else if persistence_marker.is_some() {
            text.0 = persistence_label.clone();
        }
    }
}

fn animate_active_move(
    time: Res<Time>,
    match_session: Res<MatchSession>,
    mut piece_query: Query<(&PieceVisual, &mut Transform)>,
) {
    let selected_pulse = 1.0 + 0.06 * (time.elapsed_secs() * 6.0).sin().abs();
    let last_move_pulse = 1.0 + 0.03 * (time.elapsed_secs() * 4.0).sin().abs();

    for (piece_visual, mut transform) in &mut piece_query {
        transform.scale = if Some(piece_visual.square) == match_session.selected_square {
            Vec3::splat(selected_pulse)
        } else if match_session
            .last_move
            .is_some_and(|last_move| piece_visual.square == last_move.to())
        {
            Vec3::splat(last_move_pulse)
        } else {
            Vec3::ONE
        };
    }
}

fn update_claim_draw_banner(
    match_session: Res<MatchSession>,
    mut claim_button_query: Query<(&mut Visibility, &Children), With<ClaimDrawButton>>,
    mut button_text_query: Query<&mut Text>,
) {
    if !match_session.is_changed() {
        return;
    }

    let Ok((mut visibility, children)) = claim_button_query.single_mut() else {
        return;
    };

    *visibility = if claim_draw_banner_visible(&match_session) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let Some(&text_entity) = children.first() else {
        return;
    };
    let Ok(mut button_text) = button_text_query.get_mut(text_entity) else {
        return;
    };

    button_text.0 = String::from(claim_draw_banner_label(&match_session));
}

fn handle_claim_draw_button_actions(
    interaction_query: Query<&Interaction, (With<ClaimDrawButton>, Changed<Interaction>)>,
    mut match_session: ResMut<MatchSession>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let _ = match_session.claim_draw();
        }
    }
}

fn side_label(side: Side) -> &'static str {
    match side {
        Side::White => "White",
        Side::Black => "Black",
    }
}

fn claim_draw_banner_visible(match_session: &MatchSession) -> bool {
    match_session.claimed_draw_reason().is_none()
        && !match_session.status().is_finished()
        && match_session.claimable_draw().is_claimable()
}

fn claim_draw_banner_label(match_session: &MatchSession) -> &'static str {
    if match_session.claimable_draw().threefold_repetition {
        "Claim Draw by Repetition"
    } else {
        "Claim Draw by Fifty-Move Rule"
    }
}

fn match_status_label(match_session: &MatchSession) -> String {
    if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
        return match claimed_draw_reason {
            ClaimedDrawReason::ThreefoldRepetition => {
                String::from("Draw claimed by threefold repetition.")
            }
            ClaimedDrawReason::FiftyMoveRule => {
                String::from("Draw claimed by the fifty-move rule.")
            }
        };
    }

    match match_session.status() {
        GameStatus::Ongoing {
            in_check: true,
            draw_available,
        } if draw_available.is_claimable() => {
            String::from("Check. A draw can also be claimed from this position.")
        }
        GameStatus::Ongoing {
            in_check: true,
            draw_available: _,
        } => String::from("Check."),
        GameStatus::Ongoing {
            in_check: false,
            draw_available,
        } if draw_available.is_claimable() => String::from("Draw is claimable from this position."),
        GameStatus::Ongoing { .. } => {
            if let Some(selected_square) = match_session.selected_square {
                format!("Selected {selected_square}.")
            } else {
                String::from("Select a piece to move.")
            }
        }
        GameStatus::Finished(GameOutcome::Win {
            winner,
            reason: WinReason::Checkmate,
        }) => format!("Checkmate. {} wins.", side_label(winner)),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {
            String::from("Stalemate.")
        }
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::FivefoldRepetition,
        ))) => String::from("Draw by fivefold repetition."),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::SeventyFiveMoveRule,
        ))) => String::from("Draw by the seventy-five move rule."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_and_status_labels_cover_result_and_claim_paths() {
        assert_eq!(side_label(Side::White), "White");
        assert_eq!(side_label(Side::Black), "Black");

        let mut match_session = MatchSession::start_local_match();
        assert_eq!(
            match_status_label(&match_session),
            "Select a piece to move."
        );

        match_session.selected_square =
            Some(chess_core::Square::from_algebraic("e2").expect("fixture square should parse"));
        assert_eq!(match_status_label(&match_session), "Selected e2.");

        match_session.replace_game_state(
            chess_core::GameState::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1")
                .expect("fixture FEN should parse"),
        );
        assert_eq!(match_status_label(&match_session), "Checkmate. White wins.");

        match_session.replace_game_state(
            chess_core::GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1")
                .expect("fixture FEN should parse"),
        );
        assert!(match_session.claim_draw());
        assert_eq!(
            match_status_label(&match_session),
            "Draw claimed by the fifty-move rule."
        );
    }

    #[test]
    fn draw_banner_visibility_and_copy_follow_claimable_state() {
        let mut match_session = MatchSession::start_local_match();
        assert!(!claim_draw_banner_visible(&match_session));

        match_session.replace_game_state(
            chess_core::GameState::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 1")
                .expect("fixture FEN should parse"),
        );

        assert!(claim_draw_banner_visible(&match_session));
        assert_eq!(
            claim_draw_banner_label(&match_session),
            "Claim Draw by Fifty-Move Rule"
        );
    }
}
