use bevy::prelude::*;
use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, PieceKind, WinReason};

use crate::app::AppScreenState;
use crate::match_state::{ClaimedDrawReason, MatchSession};
use crate::style::ShellTheme;

pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (configure_ambient_light, spawn_shell_camera))
            .add_systems(OnEnter(AppScreenState::Boot), advance_to_main_menu)
            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_shell_ui)
            .add_systems(OnExit(AppScreenState::MainMenu), cleanup_main_menu_ui)
            .add_systems(OnEnter(AppScreenState::MatchLoading), initialize_local_match)
            .add_systems(OnEnter(AppScreenState::MatchResult), spawn_match_result_ui)
            .add_systems(OnExit(AppScreenState::InMatch), cleanup_promotion_overlay)
            .add_systems(OnExit(AppScreenState::MatchResult), cleanup_match_result_ui)
            .add_systems(
                Update,
                (
                    orbit_camera.run_if(in_state(AppScreenState::MainMenu)),
                    sync_promotion_overlay.run_if(in_state(AppScreenState::InMatch)),
                    handle_shell_button_actions,
                    advance_to_match_result.run_if(in_state(AppScreenState::InMatch)),
                ),
            );
    }
}

#[derive(Component)]
struct MainMenuUi;

#[derive(Component)]
struct MatchResultUi;

#[derive(Component)]
struct PromotionOverlayUi;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct ShellActionButton {
    action: ShellAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellAction {
    StartLocalMatch,
    Rematch,
    ReturnToMenu,
    Promote(PieceKind),
}

#[derive(Component)]
struct ShellCamera {
    orbit_angle: f32,
}

fn configure_ambient_light(mut commands: Commands, theme: Res<ShellTheme>) {
    commands.insert_resource(AmbientLight {
        color: theme.ambient_color,
        brightness: theme.ambient_brightness,
        ..default()
    });
}

fn spawn_shell_camera(mut commands: Commands, theme: Res<ShellTheme>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            theme.camera_radius,
            theme.camera_height,
            theme.camera_radius,
        )
        .looking_at(theme.camera_focus, Vec3::Y),
        ShellCamera { orbit_angle: 0.0 },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.05, -0.85, 0.0)),
    ));

    commands.spawn((
        PointLight {
            intensity: 1_200_000.0,
            range: 30.0,
            color: theme.accent,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-6.5, 8.0, 6.0),
    ));
}

fn advance_to_main_menu(mut next_state: ResMut<NextState<AppScreenState>>) {
    next_state.set(AppScreenState::MainMenu);
}

fn spawn_shell_ui(mut commands: Commands, theme: Res<ShellTheme>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(24.0), Val::Px(24.0)),
                ..default()
            },
            MainMenuUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        ..default()
                    },
                    BackgroundColor(theme.ui_panel),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("3D Chess"),
                        TextFont {
                            font_size: 56.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
                    panel.spawn((
                        Text::new("Start a local 3D match"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                    panel.spawn((
                        Text::new(
                            "M2 begins the playable shell: start a local match now, keep chess_core \
authoritative, and leave wider shell work for later milestones.",
                        ),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));

                    spawn_action_button(
                        panel,
                        "Start Local Match",
                        theme.as_ref(),
                        ShellAction::StartLocalMatch,
                        true,
                    );
                });

            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::FlexEnd,
                        width: Val::Px(360.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.60)),
                ))
                .with_children(|panel| {
                    for line in [
                        "Match session lives in game_app and wraps chess_core",
                        "Result transitions observe domain status only",
                        "Stockfish/UCI boundary remains reserved for M4",
                    ] {
                        panel.spawn((
                            Text::new(line),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(theme.ui_text),
                        ));
                    }
                });
        });
}

fn cleanup_main_menu_ui(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUi>>) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}

fn initialize_local_match(
    mut match_session: ResMut<MatchSession>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    match_session.reset_for_local_match();
    next_state.set(AppScreenState::InMatch);
}

fn spawn_match_result_ui(
    mut commands: Commands,
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
) {
    let result_title = match_session_result_title(match_session.as_ref());
    let result_detail = match_session_result_detail(match_session.as_ref());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            MatchResultUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(520.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(18.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        ..default()
                    },
                    BackgroundColor(theme.ui_panel),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(result_title),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
                    panel.spawn((
                        Text::new(result_detail),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                    panel.spawn((
                        Text::new(
                            "Rematch resets the domain session to the starting position. \
Return to Menu keeps the shell path narrow until broader M3 flows land.",
                        ),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));

                    spawn_action_button(
                        panel,
                        "Rematch",
                        theme.as_ref(),
                        ShellAction::Rematch,
                        true,
                    );
                    spawn_action_button(
                        panel,
                        "Return to Menu",
                        theme.as_ref(),
                        ShellAction::ReturnToMenu,
                        false,
                    );
                });
        });
}

fn cleanup_match_result_ui(
    mut commands: Commands,
    result_query: Query<Entity, With<MatchResultUi>>,
) {
    for entity in &result_query {
        commands.entity(entity).despawn();
    }
}

fn sync_promotion_overlay(
    mut commands: Commands,
    promotion_overlay_query: Query<Entity, With<PromotionOverlayUi>>,
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
) {
    if let Some(pending_move) = match_session.pending_promotion_move {
        if !promotion_overlay_query.is_empty() {
            return;
        }

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.02, 0.03, 0.55)),
                PromotionOverlayUi,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(420.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(14.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(theme.ui_panel),
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("Choose Promotion"),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(theme.ui_text),
                        ));
                        panel.spawn((
                            Text::new(format!(
                                "{} -> {} requires a promotion choice.",
                                pending_move.from(),
                                pending_move.to()
                            )),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(theme.accent),
                        ));

                        for (label, promotion_piece) in [
                            ("Queen", PieceKind::Queen),
                            ("Rook", PieceKind::Rook),
                            ("Bishop", PieceKind::Bishop),
                            ("Knight", PieceKind::Knight),
                        ] {
                            spawn_action_button(
                                panel,
                                label,
                                theme.as_ref(),
                                ShellAction::Promote(promotion_piece),
                                promotion_piece == PieceKind::Queen,
                            );
                        }
                    });
            });
    } else {
        cleanup_promotion_overlay(commands, promotion_overlay_query);
    }
}

fn cleanup_promotion_overlay(
    mut commands: Commands,
    promotion_overlay_query: Query<Entity, With<PromotionOverlayUi>>,
) {
    for entity in &promotion_overlay_query {
        commands.entity(entity).despawn();
    }
}

fn handle_shell_button_actions(
    interaction_query: Query<(&Interaction, &ShellActionButton), Changed<Interaction>>,
    mut match_session: ResMut<MatchSession>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button_action.action {
            ShellAction::StartLocalMatch | ShellAction::Rematch => {
                next_state.set(AppScreenState::MatchLoading);
            }
            ShellAction::ReturnToMenu => {
                next_state.set(AppScreenState::MainMenu);
            }
            ShellAction::Promote(piece_kind) => {
                if let Some(pending_move) = match_session.pending_promotion_move {
                    let _ = match_session.apply_move(chess_core::Move::with_promotion(
                        pending_move.from(),
                        pending_move.to(),
                        piece_kind,
                    ));
                }
            }
        }
    }
}

fn advance_to_match_result(
    match_session: Res<MatchSession>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    if match_session.is_finished() {
        next_state.set(AppScreenState::MatchResult);
    }
}

fn orbit_camera(
    time: Res<Time>,
    theme: Res<ShellTheme>,
    mut camera_query: Query<(&mut Transform, &mut ShellCamera)>,
) {
    for (mut transform, mut shell_camera) in &mut camera_query {
        shell_camera.orbit_angle += time.delta_secs() * theme.orbit_speed;

        let x = theme.camera_focus.x + shell_camera.orbit_angle.cos() * theme.camera_radius;
        let z = theme.camera_focus.z + shell_camera.orbit_angle.sin() * theme.camera_radius;

        transform.translation = Vec3::new(x, theme.camera_height, z);
        transform.look_at(theme.camera_focus, Vec3::Y);
    }
}

fn spawn_action_button(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    theme: &ShellTheme,
    action: ShellAction,
    accent: bool,
) {
    let background = if accent {
        theme.accent
    } else {
        Color::srgba(0.10, 0.13, 0.18, 0.92)
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(8.0)),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(background),
            ShellActionButton { action },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(theme.ui_text),
            ));
        });
}

fn match_session_result_title(match_session: &MatchSession) -> String {
    if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
        return match claimed_draw_reason {
            ClaimedDrawReason::ThreefoldRepetition => {
                String::from("Draw Claimed by Repetition")
            }
            ClaimedDrawReason::FiftyMoveRule => String::from("Draw Claimed by Fifty-Move Rule"),
        };
    }

    match match_session.status() {
        chess_core::GameStatus::Ongoing { .. } => String::from("Match Complete"),
        chess_core::GameStatus::Finished(GameOutcome::Win {
            winner,
            reason: WinReason::Checkmate,
        }) => match winner {
            chess_core::Side::White => String::from("White Wins"),
            chess_core::Side::Black => String::from("Black Wins"),
        },
        chess_core::GameStatus::Finished(GameOutcome::Draw(_)) => String::from("Draw"),
    }
}

fn match_session_result_detail(match_session: &MatchSession) -> String {
    if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
        return match claimed_draw_reason {
            ClaimedDrawReason::ThreefoldRepetition => {
                String::from("Threefold repetition was claimed from the in-match HUD.")
            }
            ClaimedDrawReason::FiftyMoveRule => {
                String::from("The fifty-move rule was claimed from the in-match HUD.")
            }
        };
    }

    match match_session.status() {
        chess_core::GameStatus::Ongoing { .. } => String::from(
            "The shell can now route into match results when chess_core reports a terminal state.",
        ),
        chess_core::GameStatus::Finished(GameOutcome::Win {
            reason: WinReason::Checkmate,
            ..
        }) => String::from("Checkmate detected by chess_core."),
        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {
            String::from("Stalemate detected by chess_core.")
        }
        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::FivefoldRepetition,
        ))) => String::from("Fivefold repetition detected by chess_core."),
        chess_core::GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::SeventyFiveMoveRule,
        ))) => String::from("Seventy-five move rule detected by chess_core."),
    }
}
