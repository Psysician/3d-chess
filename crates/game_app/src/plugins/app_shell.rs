// Shell handlers translate semantic navigation, save, settings, and
// confirmation actions into the existing message flow.
// (refs: DL-003, DL-005)

//! Presentation layer for the coarse app shell.
//! Main menu, pause overlay, and results render from modal resources while match launch still funnels through MatchLoading. (ref: DL-001) (ref: DL-007)

use bevy::prelude::*;
use chess_core::PieceKind;

use super::app_shell_logic;
use super::input::apply_promotion_choice;
use super::menu::{
    ConfirmationKind, MenuAction, MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState,
};
use super::save_load::{SaveLoadRequest, SaveLoadState};
use crate::app::AppScreenState;
use crate::automation::{
    AutomationConfirmationKind, AutomationError, AutomationNavigationAction, AutomationResult,
    AutomationSaveAction, AutomationSettingsAction,
};
use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
use crate::style::ShellTheme;

pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (configure_ambient_light, spawn_shell_camera))
            .add_systems(OnEnter(AppScreenState::Boot), advance_to_main_menu)
            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_main_menu_ui)
            .add_systems(
                OnEnter(AppScreenState::MatchLoading),
                resolve_match_launch_intent,
            )
            .add_systems(OnEnter(AppScreenState::MatchResult), spawn_match_result_ui)
            .add_systems(OnExit(AppScreenState::MainMenu), cleanup_shell_overlay)
            .add_systems(OnExit(AppScreenState::InMatch), cleanup_shell_overlay)
            .add_systems(OnExit(AppScreenState::InMatch), cleanup_promotion_overlay)
            .add_systems(OnExit(AppScreenState::MatchResult), cleanup_match_result_ui)
            .add_systems(
                Update,
                (
                    orbit_camera,
                    refresh_shell_overlay,
                    sync_promotion_overlay.run_if(in_state(AppScreenState::InMatch)),
                    handle_shell_button_actions,
                    advance_to_match_result.run_if(in_state(AppScreenState::InMatch)),
                ),
            );
    }
}

#[derive(Component)]
struct ShellOverlayUi;

#[derive(Component)]
struct MatchResultUi;

#[derive(Component)]
struct PromotionOverlayUi;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct ShellActionButton {
    action: ShellAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellAction {
    OpenSetup,
    BackToSetup,
    StartNewMatch,
    OpenLoadList,
    OpenSettings,
    ResumeRecovery,
    ResumeMatch,
    ReturnToMenu,
    Rematch,
    SaveManual,
    OverwriteSelectedSave,
    LoadSelected,
    DeleteSelected,
    SelectSave(String),
    CycleRecoveryPolicy,
    ToggleDisplayMode,
    ToggleConfirmation(ConfirmationKind),
    CancelModal,
    Confirm(ConfirmationKind),
    Promote(PieceKind),
}

impl From<ConfirmationKind> for AutomationConfirmationKind {
    fn from(value: ConfirmationKind) -> Self {
        match value {
            ConfirmationKind::AbandonMatch => Self::AbandonMatch,
            ConfirmationKind::DeleteSave => Self::DeleteSave,
            ConfirmationKind::OverwriteSave => Self::OverwriteSave,
        }
    }
}

impl From<AutomationConfirmationKind> for ConfirmationKind {
    fn from(value: AutomationConfirmationKind) -> Self {
        match value {
            AutomationConfirmationKind::AbandonMatch => Self::AbandonMatch,
            AutomationConfirmationKind::DeleteSave => Self::DeleteSave,
            AutomationConfirmationKind::OverwriteSave => Self::OverwriteSave,
        }
    }
}

fn navigation_from_shell(action: &ShellAction) -> Option<AutomationNavigationAction> {
    match action {
        ShellAction::OpenSetup => Some(AutomationNavigationAction::OpenSetup),
        ShellAction::BackToSetup => Some(AutomationNavigationAction::BackToSetup),
        ShellAction::StartNewMatch => Some(AutomationNavigationAction::StartNewMatch),
        ShellAction::OpenLoadList => Some(AutomationNavigationAction::OpenLoadList),
        ShellAction::OpenSettings => Some(AutomationNavigationAction::OpenSettings),
        ShellAction::ResumeRecovery => Some(AutomationNavigationAction::ResumeRecovery),
        ShellAction::ResumeMatch => Some(AutomationNavigationAction::ResumeMatch),
        ShellAction::ReturnToMenu => Some(AutomationNavigationAction::ReturnToMenu),
        ShellAction::Rematch => Some(AutomationNavigationAction::Rematch),
        ShellAction::CancelModal => Some(AutomationNavigationAction::CancelModal),
        _ => None,
    }
}

fn save_from_shell(action: &ShellAction) -> Option<AutomationSaveAction> {
    match action {
        ShellAction::SaveManual => Some(AutomationSaveAction::SaveManual { label: None }),
        ShellAction::OverwriteSelectedSave => Some(AutomationSaveAction::OverwriteSelected),
        ShellAction::LoadSelected => Some(AutomationSaveAction::LoadSelected),
        ShellAction::DeleteSelected => Some(AutomationSaveAction::DeleteSelected),
        ShellAction::SelectSave(slot_id) => Some(AutomationSaveAction::SelectSlot {
            slot_id: slot_id.clone(),
        }),
        _ => None,
    }
}

fn settings_from_shell(action: &ShellAction) -> Option<AutomationSettingsAction> {
    match action {
        ShellAction::CycleRecoveryPolicy => Some(AutomationSettingsAction::CycleRecoveryPolicy),
        ShellAction::ToggleDisplayMode => Some(AutomationSettingsAction::ToggleDisplayMode),
        ShellAction::ToggleConfirmation(kind) => {
            Some(AutomationSettingsAction::ToggleConfirmation {
                kind: (*kind).into(),
            })
        }
        _ => None,
    }
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

fn spawn_main_menu_ui(
    mut commands: Commands,
    theme: Res<ShellTheme>,
    menu_state: Res<ShellMenuState>,
    save_state: Res<SaveLoadState>,
    recovery: Res<RecoveryBannerState>,
) {
    if matches!(menu_state.panel, MenuPanel::Home) {
        build_main_menu_ui(&mut commands, theme.as_ref(), recovery.as_ref());
    } else {
        build_setup_ui(
            &mut commands,
            theme.as_ref(),
            menu_state.as_ref(),
            save_state.as_ref(),
            recovery.as_ref(),
            false,
        );
    }
}

fn build_main_menu_ui(commands: &mut Commands, theme: &ShellTheme, recovery: &RecoveryBannerState) {
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
            ShellOverlayUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(460.0),
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
                        Text::new("M3 completes the local product shell"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                    panel.spawn((
                        Text::new(
                            "Open local match setup, manage saves, and resume interrupted sessions without widening top-level routing.",
                        ),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
                    spawn_action_button(
                        panel,
                        "Local Match Setup",
                        theme,
                        ShellAction::OpenSetup,
                        true,
                    );
                    if recovery.available {
                        spawn_action_button(
                            panel,
                            "Resume Interrupted Match",
                            theme,
                            ShellAction::ResumeRecovery,
                            false,
                        );
                    }
                });
        });
}

/// Renders the setup/load/settings surface for both the main menu and the in-match pause overlay.
/// The panel stays modal so setup, load, startup recovery, destructive confirmations, and display mode do not add more top-level app states. (ref: DL-001) (ref: DL-005)
fn build_setup_ui(
    commands: &mut Commands,
    theme: &ShellTheme,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    recovery: &RecoveryBannerState,
    paused: bool,
) {
    let (title, subtitle) = setup_copy(paused);

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
            ShellOverlayUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(560.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(22.0)),
                        ..default()
                    },
                    BackgroundColor(theme.ui_panel),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
                    panel.spawn((
                        Text::new(subtitle),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                    spawn_setup_status(panel, theme, menu_state, save_state, recovery);
                    spawn_setup_panel_actions(
                        panel, theme, menu_state, save_state, recovery, paused,
                    );
                    spawn_confirmation_actions(panel, theme, menu_state.confirmation);
                });
        });
}

fn setup_copy(paused: bool) -> (&'static str, &'static str) {
    if paused {
        (
            "Paused",
            "Save, load, or abandon without bypassing recovery safeguards.",
        )
    } else {
        (
            "Local Match Setup",
            "Choose how the next local session should begin.",
        )
    }
}

fn spawn_setup_status(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    recovery: &RecoveryBannerState,
) {
    if let Some(status) = app_shell_logic::effective_shell_status(menu_state, save_state, recovery)
    {
        panel.spawn((
            Text::new(status),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(theme.ui_text),
        ));
    }
}

fn spawn_setup_panel_actions(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    recovery: &RecoveryBannerState,
    paused: bool,
) {
    match menu_state.panel {
        MenuPanel::Home | MenuPanel::Setup => {
            spawn_setup_home_actions(panel, theme, recovery, paused);
        }
        MenuPanel::LoadList => {
            spawn_load_list_actions(panel, theme, menu_state, save_state, paused);
        }
        MenuPanel::Settings => {
            spawn_settings_actions(panel, theme, save_state);
        }
    }
}

fn spawn_setup_home_actions(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    recovery: &RecoveryBannerState,
    paused: bool,
) {
    if paused {
        spawn_action_button(panel, "Resume Match", theme, ShellAction::ResumeMatch, true);
        spawn_action_button(
            panel,
            "Create Manual Save",
            theme,
            ShellAction::SaveManual,
            false,
        );
    } else {
        spawn_action_button(
            panel,
            "Start New Match",
            theme,
            ShellAction::StartNewMatch,
            true,
        );
    }

    spawn_action_button(
        panel,
        "Open Save Slots",
        theme,
        ShellAction::OpenLoadList,
        false,
    );
    spawn_action_button(panel, "Settings", theme, ShellAction::OpenSettings, false);

    if recovery.available {
        spawn_action_button(
            panel,
            "Resume Interrupted Match",
            theme,
            ShellAction::ResumeRecovery,
            false,
        );
    }

    spawn_action_button(
        panel,
        if paused {
            "Return to Main Menu"
        } else {
            "Back to Main Menu"
        },
        theme,
        ShellAction::ReturnToMenu,
        false,
    );
}

fn spawn_load_list_actions(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    paused: bool,
) {
    if save_state.manual_saves.is_empty() {
        panel.spawn((
            Text::new("No manual saves are available yet."),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(theme.ui_text),
        ));
    } else {
        for save in &save_state.manual_saves {
            let selected = menu_state.selected_save.as_deref() == Some(save.slot_id.as_str());
            let label = if selected {
                format!("> {}", save.label)
            } else {
                save.label.clone()
            };
            spawn_action_button(
                panel,
                &label,
                theme,
                ShellAction::SelectSave(save.slot_id.clone()),
                selected,
            );
        }
    }

    spawn_action_button(
        panel,
        "Load Selected Save",
        theme,
        ShellAction::LoadSelected,
        true,
    );
    if paused {
        spawn_action_button(
            panel,
            "Overwrite Selected Save",
            theme,
            ShellAction::OverwriteSelectedSave,
            false,
        );
    }
    spawn_action_button(
        panel,
        "Delete Selected Save",
        theme,
        ShellAction::DeleteSelected,
        false,
    );
    spawn_action_button(panel, "Back", theme, ShellAction::BackToSetup, false);
}

fn spawn_settings_actions(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    save_state: &SaveLoadState,
) {
    panel.spawn((
        Text::new(format!(
            "Startup recovery: {}",
            app_shell_logic::recovery_policy_label(save_state.settings.recovery_policy)
        )),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(theme.ui_text),
    ));
    spawn_action_button(
        panel,
        "Cycle Startup Recovery",
        theme,
        ShellAction::CycleRecoveryPolicy,
        false,
    );
    panel.spawn((
        Text::new(format!(
            "Display mode: {}",
            app_shell_logic::display_mode_label(save_state.settings.display_mode)
        )),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(theme.ui_text),
    ));
    spawn_action_button(
        panel,
        "Toggle Display Mode",
        theme,
        ShellAction::ToggleDisplayMode,
        false,
    );
    spawn_action_button(
        panel,
        &app_shell_logic::toggle_label(
            "Confirm menu abandon",
            save_state.settings.confirm_actions.abandon_match,
        ),
        theme,
        ShellAction::ToggleConfirmation(ConfirmationKind::AbandonMatch),
        false,
    );
    spawn_action_button(
        panel,
        &app_shell_logic::toggle_label(
            "Confirm save delete",
            save_state.settings.confirm_actions.delete_save,
        ),
        theme,
        ShellAction::ToggleConfirmation(ConfirmationKind::DeleteSave),
        false,
    );
    spawn_action_button(
        panel,
        &app_shell_logic::toggle_label(
            "Confirm save overwrite",
            save_state.settings.confirm_actions.overwrite_save,
        ),
        theme,
        ShellAction::ToggleConfirmation(ConfirmationKind::OverwriteSave),
        false,
    );
    spawn_action_button(panel, "Back", theme, ShellAction::BackToSetup, false);
}

fn spawn_confirmation_actions(
    panel: &mut ChildSpawnerCommands<'_>,
    theme: &ShellTheme,
    confirmation: Option<ConfirmationKind>,
) {
    let Some(kind) = confirmation else {
        return;
    };

    let (headline, detail) = app_shell_logic::confirmation_copy(kind);
    panel.spawn((
        Text::new(headline),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.accent),
    ));
    panel.spawn((
        Text::new(detail),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.ui_text),
    ));
    spawn_action_button(panel, "Confirm", theme, ShellAction::Confirm(kind), true);
    spawn_action_button(panel, "Cancel", theme, ShellAction::CancelModal, false);
}

/// Rebuilds whichever shell overlay matches the coarse route and modal menu context.
/// Rendering from resources keeps UI nodes disposable and leaves state ownership in dedicated shell resources. (ref: DL-001) (ref: DL-007)
fn refresh_shell_overlay(
    state: Res<State<AppScreenState>>,
    theme: Res<ShellTheme>,
    menu_state: Res<ShellMenuState>,
    save_state: Res<SaveLoadState>,
    recovery: Res<RecoveryBannerState>,
    overlay_query: Query<Entity, With<ShellOverlayUi>>,
    mut commands: Commands,
) {
    let render_main_menu = *state.get() == AppScreenState::MainMenu;
    let render_pause_overlay = *state.get() == AppScreenState::InMatch
        && menu_state.context == MenuContext::InMatchOverlay;

    if !render_main_menu && !render_pause_overlay {
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    if !overlay_query.is_empty()
        && !menu_state.is_changed()
        && !save_state.is_changed()
        && !recovery.is_changed()
    {
        return;
    }

    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }

    if render_main_menu {
        if matches!(menu_state.panel, MenuPanel::Home) {
            build_main_menu_ui(&mut commands, theme.as_ref(), recovery.as_ref());
        } else {
            build_setup_ui(
                &mut commands,
                theme.as_ref(),
                menu_state.as_ref(),
                save_state.as_ref(),
                recovery.as_ref(),
                false,
            );
        }
        return;
    }

    build_setup_ui(
        &mut commands,
        theme.as_ref(),
        menu_state.as_ref(),
        save_state.as_ref(),
        recovery.as_ref(),
        true,
    );
}

fn cleanup_shell_overlay(
    mut commands: Commands,
    overlay_query: Query<Entity, With<ShellOverlayUi>>,
) {
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }
}

/// Consumes the explicit launch intent before entering `InMatch`.
/// Match loading either resets the domain session or hydrates a pending snapshot, but it never guesses which path the user meant. (ref: DL-001)
fn resolve_match_launch_intent(
    mut match_session: ResMut<MatchSession>,
    mut launch_intent: ResMut<MatchLaunchIntent>,
    mut pending_snapshot: ResMut<PendingLoadedSnapshot>,
    mut menu_state: ResMut<ShellMenuState>,
    mut next_state: ResMut<NextState<AppScreenState>>,
) {
    match *launch_intent {
        MatchLaunchIntent::NewLocalMatch | MatchLaunchIntent::Rematch => {
            match_session.reset_for_local_match();
        }
        MatchLaunchIntent::LoadManual | MatchLaunchIntent::ResumeRecovery => {
            let Some(snapshot) = pending_snapshot.0.take() else {
                menu_state.status_line = Some(String::from("No saved session was ready to load."));
                menu_state.context = MenuContext::MainMenu;
                menu_state.panel = MenuPanel::Setup;
                next_state.set(AppScreenState::MainMenu);
                return;
            };
            *match_session = MatchSession::restore_from_snapshot(&snapshot);
        }
    }

    *launch_intent = MatchLaunchIntent::NewLocalMatch;
    menu_state.context = MenuContext::MainMenu;
    menu_state.panel = MenuPanel::Setup;
    menu_state.confirmation = None;
    next_state.set(AppScreenState::InMatch);
}

fn spawn_match_result_ui(
    mut commands: Commands,
    match_session: Res<MatchSession>,
    theme: Res<ShellTheme>,
) {
    let result_title = app_shell_logic::match_session_result_title(
        match_session.status(),
        match_session.claimed_draw_reason(),
    );
    let result_detail = app_shell_logic::match_session_result_detail(
        match_session.status(),
        match_session.claimed_draw_reason(),
    );

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
                    spawn_action_button(
                        panel,
                        "Rematch",
                        theme.as_ref(),
                        ShellAction::Rematch,
                        true,
                    );
                    spawn_action_button(
                        panel,
                        "Return to Main Menu",
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
    state: Res<State<AppScreenState>>,
    menu_state: Res<ShellMenuState>,
    save_state: Res<SaveLoadState>,
    mut menu_actions: MessageWriter<MenuAction>,
    mut save_requests: MessageWriter<SaveLoadRequest>,
    mut match_session_mut: ResMut<MatchSession>,
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button_action.action {
            action if navigation_from_shell(action).is_some() => {
                handle_navigation_action(
                    navigation_from_shell(action).unwrap(),
                    *state.get(),
                    menu_state.as_ref(),
                    save_state.as_ref(),
                    &mut menu_actions,
                    &mut save_requests,
                );
            }
            action if save_from_shell(action).is_some() => {
                let _ = handle_save_slot_action(
                    &save_from_shell(action).unwrap(),
                    menu_state.as_ref(),
                    save_state.as_ref(),
                    &mut menu_actions,
                    &mut save_requests,
                    match_session_mut.as_ref(),
                );
            }
            action if settings_from_shell(action).is_some() => {
                handle_settings_action(&settings_from_shell(action).unwrap(), &mut save_requests);
            }
            ShellAction::Confirm(kind) => {
                let _ = handle_confirmation_action(
                    (*kind).into(),
                    menu_state.as_ref(),
                    save_state.as_ref(),
                    &mut menu_actions,
                    &mut save_requests,
                );
            }
            ShellAction::Promote(piece_kind) => {
                let _ = handle_promotion_action(*piece_kind, match_session_mut.as_mut());
            }
            _ => {}
        }
    }
}

pub(crate) fn handle_navigation_action(
    action: AutomationNavigationAction,
    state: AppScreenState,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    menu_actions: &mut MessageWriter<MenuAction>,
    save_requests: &mut MessageWriter<SaveLoadRequest>,
) {
    match action {
        AutomationNavigationAction::OpenSetup => {
            menu_actions.write(MenuAction::OpenSetup);
        }
        AutomationNavigationAction::BackToSetup => {
            menu_actions.write(MenuAction::BackToSetup);
        }
        AutomationNavigationAction::OpenLoadList => {
            menu_actions.write(MenuAction::OpenLoadList);
        }
        AutomationNavigationAction::OpenSettings => {
            menu_actions.write(MenuAction::OpenSettings);
        }
        AutomationNavigationAction::StartNewMatch => {
            menu_actions.write(MenuAction::StartNewMatch);
        }
        AutomationNavigationAction::ResumeRecovery => {
            save_requests.write(SaveLoadRequest::ResumeRecovery);
        }
        AutomationNavigationAction::PauseMatch => {
            menu_actions.write(MenuAction::PauseMatch);
        }
        AutomationNavigationAction::ResumeMatch => {
            menu_actions.write(MenuAction::ResumeMatch);
        }
        AutomationNavigationAction::ReturnToMenu => {
            request_return_to_menu(state, menu_state, save_state, menu_actions, save_requests);
        }
        AutomationNavigationAction::Rematch => {
            menu_actions.write(MenuAction::Rematch);
        }
        AutomationNavigationAction::CancelModal => {
            menu_actions.write(MenuAction::CancelModal);
        }
    }
}

fn request_return_to_menu(
    state: AppScreenState,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    menu_actions: &mut MessageWriter<MenuAction>,
    save_requests: &mut MessageWriter<SaveLoadRequest>,
) {
    let abandoning_live_match =
        app_shell_logic::return_to_menu_abandons_active_match(state, menu_state);
    if abandoning_live_match && save_state.settings.confirm_actions.abandon_match {
        menu_actions.write(MenuAction::RequestConfirmation(
            ConfirmationKind::AbandonMatch,
        ));
    } else if abandoning_live_match {
        save_requests.write(SaveLoadRequest::AbandonMatchAndReturnToMenu);
    } else {
        menu_actions.write(MenuAction::ReturnToMenu);
    }
}

pub(crate) fn handle_save_slot_action(
    action: &AutomationSaveAction,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    menu_actions: &mut MessageWriter<MenuAction>,
    save_requests: &mut MessageWriter<SaveLoadRequest>,
    match_session: &MatchSession,
) -> AutomationResult<()> {
    match action {
        AutomationSaveAction::RefreshIndex => {
            save_requests.write(SaveLoadRequest::RefreshIndex);
        }
        AutomationSaveAction::SaveManual { label } => {
            save_requests.write(SaveLoadRequest::SaveManual {
                label: label
                    .clone()
                    .unwrap_or_else(|| app_shell_logic::derive_save_label(match_session.last_move)),
                slot_id: None,
            });
        }
        AutomationSaveAction::OverwriteSelected => {
            if let Some(selected) = app_shell_logic::selected_save_summary(menu_state, save_state) {
                if save_state.settings.confirm_actions.overwrite_save {
                    menu_actions.write(MenuAction::RequestConfirmation(
                        ConfirmationKind::OverwriteSave,
                    ));
                } else {
                    save_requests.write(SaveLoadRequest::SaveManual {
                        label: selected.label.clone(),
                        slot_id: Some(selected.slot_id.clone()),
                    });
                }
            } else {
                return Err(AutomationError::SaveSelectionRequired);
            }
        }
        AutomationSaveAction::LoadSelected => {
            if let Some(slot_id) = menu_state.selected_save.clone() {
                save_requests.write(SaveLoadRequest::LoadManual { slot_id });
            } else {
                return Err(AutomationError::SaveSelectionRequired);
            }
        }
        AutomationSaveAction::DeleteSelected => {
            if let Some(slot_id) = menu_state.selected_save.clone() {
                if save_state.settings.confirm_actions.delete_save {
                    menu_actions.write(MenuAction::RequestConfirmation(
                        ConfirmationKind::DeleteSave,
                    ));
                } else {
                    save_requests.write(SaveLoadRequest::DeleteManual { slot_id });
                }
            } else {
                return Err(AutomationError::SaveSelectionRequired);
            }
        }
        AutomationSaveAction::SelectSlot { slot_id } => {
            menu_actions.write(MenuAction::SelectSave(slot_id.clone()));
        }
    }
    Ok(())
}

pub(crate) fn handle_settings_action(
    action: &AutomationSettingsAction,
    save_requests: &mut MessageWriter<SaveLoadRequest>,
) {
    match action {
        AutomationSettingsAction::CycleRecoveryPolicy => {
            save_requests.write(SaveLoadRequest::CycleRecoveryPolicy);
        }
        AutomationSettingsAction::ToggleDisplayMode => {
            save_requests.write(SaveLoadRequest::ToggleDisplayMode);
        }
        AutomationSettingsAction::ToggleConfirmation { kind } => {
            save_requests.write(SaveLoadRequest::ToggleConfirmation((*kind).into()));
        }
    }
}

pub(crate) fn handle_confirmation_action(
    kind: AutomationConfirmationKind,
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    menu_actions: &mut MessageWriter<MenuAction>,
    save_requests: &mut MessageWriter<SaveLoadRequest>,
) -> AutomationResult<()> {
    match ConfirmationKind::from(kind) {
        ConfirmationKind::AbandonMatch => {
            save_requests.write(SaveLoadRequest::AbandonMatchAndReturnToMenu);
        }
        ConfirmationKind::DeleteSave => {
            if let Some(slot_id) = menu_state.selected_save.clone() {
                save_requests.write(SaveLoadRequest::DeleteManual { slot_id });
            } else {
                return Err(AutomationError::SaveSelectionRequired);
            }
            menu_actions.write(MenuAction::CancelModal);
        }
        ConfirmationKind::OverwriteSave => {
            if let Some(selected) = app_shell_logic::selected_save_summary(menu_state, save_state) {
                save_requests.write(SaveLoadRequest::SaveManual {
                    label: selected.label.clone(),
                    slot_id: Some(selected.slot_id.clone()),
                });
            } else {
                return Err(AutomationError::SaveSelectionRequired);
            }
            menu_actions.write(MenuAction::CancelModal);
        }
    }
    Ok(())
}

fn handle_promotion_action(
    piece_kind: PieceKind,
    match_session: &mut MatchSession,
) -> AutomationResult<()> {
    apply_promotion_choice(match_session, piece_kind)
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
    state: Res<State<AppScreenState>>,
    theme: Res<ShellTheme>,
    mut camera_query: Query<(&mut Transform, &mut ShellCamera)>,
) {
    if *state.get() != AppScreenState::MainMenu {
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::SystemState;
    use chess_core::{GameState, Move, Square};
    use chess_persistence::{
        ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SaveKind, SavedSessionSummary,
        ShellSettings,
    };

    fn drain_messages<T: Message>(world: &mut World) -> Vec<T> {
        world.resource_mut::<Messages<T>>().drain().collect()
    }

    #[test]
    fn return_to_menu_from_main_menu_setup_preserves_recovery_state() {
        let menu_state = ShellMenuState {
            panel: MenuPanel::Setup,
            context: MenuContext::MainMenu,
            ..Default::default()
        };

        assert!(!app_shell_logic::return_to_menu_abandons_active_match(
            AppScreenState::MainMenu,
            &menu_state,
        ));
    }

    #[test]
    fn return_to_menu_from_in_match_overlay_abandons_live_match() {
        let menu_state = ShellMenuState {
            panel: MenuPanel::Setup,
            context: MenuContext::InMatchOverlay,
            ..Default::default()
        };

        assert!(app_shell_logic::return_to_menu_abandons_active_match(
            AppScreenState::InMatch,
            &menu_state,
        ));
    }

    #[test]
    fn effective_shell_status_ignores_hidden_recovery_labels() {
        let recovery = RecoveryBannerState {
            available: false,
            dirty: false,
            label: Some(String::from("Interrupted Session")),
        };

        assert_eq!(
            app_shell_logic::effective_shell_status(
                &ShellMenuState::default(),
                &SaveLoadState::default(),
                &recovery,
            ),
            None
        );
    }

    #[test]
    fn navigation_actions_dispatch_menu_and_save_messages() {
        type WriterState<'w, 's> = SystemState<(
            MessageWriter<'w, MenuAction>,
            MessageWriter<'w, SaveLoadRequest>,
        )>;

        let mut world = World::new();
        world.init_resource::<Messages<MenuAction>>();
        world.init_resource::<Messages<SaveLoadRequest>>();
        let mut writers: WriterState<'_, '_> = SystemState::new(&mut world);

        let menu_state = ShellMenuState {
            context: MenuContext::InMatchOverlay,
            ..Default::default()
        };
        let save_state = SaveLoadState {
            settings: ShellSettings {
                recovery_policy: RecoveryStartupPolicy::Ask,
                confirm_actions: ConfirmActionSettings {
                    overwrite_save: true,
                    delete_save: true,
                    abandon_match: true,
                },
                display_mode: DisplayMode::Windowed,
            },
            ..Default::default()
        };

        {
            let (mut menu_actions, mut save_requests) = writers.get_mut(&mut world);
            handle_navigation_action(
                AutomationNavigationAction::OpenSetup,
                AppScreenState::MainMenu,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
            );
            handle_navigation_action(
                AutomationNavigationAction::ResumeRecovery,
                AppScreenState::MainMenu,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
            );
            handle_navigation_action(
                AutomationNavigationAction::ReturnToMenu,
                AppScreenState::InMatch,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
            );
        }

        let menu_messages = drain_messages::<MenuAction>(&mut world);
        let save_messages = drain_messages::<SaveLoadRequest>(&mut world);
        assert!(menu_messages.contains(&MenuAction::OpenSetup));
        assert!(menu_messages.contains(&MenuAction::RequestConfirmation(
            ConfirmationKind::AbandonMatch
        )));
        assert!(save_messages.contains(&SaveLoadRequest::ResumeRecovery));
    }

    #[test]
    fn save_slot_and_confirmation_actions_dispatch_expected_requests() {
        type WriterState<'w, 's> = SystemState<(
            MessageWriter<'w, MenuAction>,
            MessageWriter<'w, SaveLoadRequest>,
        )>;

        let mut world = World::new();
        world.init_resource::<Messages<MenuAction>>();
        world.init_resource::<Messages<SaveLoadRequest>>();
        let mut writers: WriterState<'_, '_> = SystemState::new(&mut world);

        let menu_state = ShellMenuState {
            selected_save: Some(String::from("slot-a")),
            ..Default::default()
        };
        let save_state = SaveLoadState {
            manual_saves: vec![SavedSessionSummary {
                slot_id: String::from("slot-a"),
                label: String::from("Slot A"),
                created_at_utc: None,
                save_kind: SaveKind::Manual,
            }],
            settings: ShellSettings {
                recovery_policy: RecoveryStartupPolicy::Ask,
                confirm_actions: ConfirmActionSettings {
                    overwrite_save: true,
                    delete_save: false,
                    abandon_match: true,
                },
                display_mode: DisplayMode::Windowed,
            },
            ..Default::default()
        };
        let match_session = MatchSession::start_local_match();

        {
            let (mut menu_actions, mut save_requests) = writers.get_mut(&mut world);
            handle_save_slot_action(
                &AutomationSaveAction::SaveManual { label: None },
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
                &match_session,
            )
            .expect("save manual action should be routable");
            handle_save_slot_action(
                &AutomationSaveAction::OverwriteSelected,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
                &match_session,
            )
            .expect("overwrite action should be routable");
            handle_save_slot_action(
                &AutomationSaveAction::DeleteSelected,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
                &match_session,
            )
            .expect("delete action should be routable");
            handle_save_slot_action(
                &AutomationSaveAction::LoadSelected,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
                &match_session,
            )
            .expect("load action should be routable");
            handle_confirmation_action(
                AutomationConfirmationKind::DeleteSave,
                &menu_state,
                &save_state,
                &mut menu_actions,
                &mut save_requests,
            )
            .expect("confirmation action should be routable");
        }

        let menu_messages = drain_messages::<MenuAction>(&mut world);
        let save_messages = drain_messages::<SaveLoadRequest>(&mut world);
        assert!(menu_messages.contains(&MenuAction::RequestConfirmation(
            ConfirmationKind::OverwriteSave
        )));
        assert!(menu_messages.contains(&MenuAction::CancelModal));
        assert!(save_messages.contains(&SaveLoadRequest::LoadManual {
            slot_id: String::from("slot-a")
        }));
        assert!(save_messages.contains(&SaveLoadRequest::DeleteManual {
            slot_id: String::from("slot-a")
        }));
        assert!(save_messages.contains(&SaveLoadRequest::SaveManual {
            label: String::from("Local Match Save"),
            slot_id: None
        }));
    }

    #[test]
    fn settings_actions_and_launch_resolution_cover_remaining_shell_branches() {
        type SaveWriterState<'w, 's> = SystemState<(MessageWriter<'w, SaveLoadRequest>,)>;

        let mut world = World::new();
        world.init_resource::<Messages<SaveLoadRequest>>();
        let mut writers: SaveWriterState<'_, '_> = SystemState::new(&mut world);

        {
            let (mut save_requests,) = writers.get_mut(&mut world);
            handle_settings_action(
                &AutomationSettingsAction::CycleRecoveryPolicy,
                &mut save_requests,
            );
            handle_settings_action(
                &AutomationSettingsAction::ToggleDisplayMode,
                &mut save_requests,
            );
            handle_settings_action(
                &AutomationSettingsAction::ToggleConfirmation {
                    kind: AutomationConfirmationKind::DeleteSave,
                },
                &mut save_requests,
            );
        }
        let save_messages = drain_messages::<SaveLoadRequest>(&mut world);
        assert!(save_messages.contains(&SaveLoadRequest::CycleRecoveryPolicy));
        assert!(save_messages.contains(&SaveLoadRequest::ToggleDisplayMode));
        assert!(save_messages.contains(&SaveLoadRequest::ToggleConfirmation(
            ConfirmationKind::DeleteSave
        )));

        type LaunchState<'w, 's> = SystemState<(
            ResMut<'w, MatchSession>,
            ResMut<'w, MatchLaunchIntent>,
            ResMut<'w, PendingLoadedSnapshot>,
            ResMut<'w, ShellMenuState>,
            ResMut<'w, NextState<AppScreenState>>,
        )>;

        let mut world = World::new();
        world.insert_resource(MatchSession::start_local_match());
        world.insert_resource(MatchLaunchIntent::LoadManual);
        world.insert_resource(PendingLoadedSnapshot(None));
        world.insert_resource(ShellMenuState::default());
        world.insert_resource(NextState::<AppScreenState>::default());
        let mut launch_state: LaunchState<'_, '_> = SystemState::new(&mut world);
        {
            let (match_session, launch_intent, pending_snapshot, menu_state, next_state) =
                launch_state.get_mut(&mut world);
            resolve_match_launch_intent(
                match_session,
                launch_intent,
                pending_snapshot,
                menu_state,
                next_state,
            );
        }
        assert_eq!(
            world.resource::<ShellMenuState>().status_line.as_deref(),
            Some("No saved session was ready to load.")
        );

        let snapshot =
            MatchSession::start_local_match().to_snapshot(chess_persistence::SnapshotMetadata {
                label: String::from("Loaded"),
                created_at_utc: None,
                updated_at_utc: None,
                notes: None,
                save_kind: SaveKind::Manual,
                session_id: String::from("slot-a"),
                recovery_key: None,
            });
        let mut world = World::new();
        world.insert_resource(MatchSession::start_local_match());
        world.insert_resource(MatchLaunchIntent::LoadManual);
        world.insert_resource(PendingLoadedSnapshot(Some(snapshot)));
        world.insert_resource(ShellMenuState {
            panel: MenuPanel::LoadList,
            context: MenuContext::InMatchOverlay,
            confirmation: Some(ConfirmationKind::DeleteSave),
            ..Default::default()
        });
        world.insert_resource(NextState::<AppScreenState>::default());
        let mut launch_state: LaunchState<'_, '_> = SystemState::new(&mut world);
        {
            let (match_session, launch_intent, pending_snapshot, menu_state, next_state) =
                launch_state.get_mut(&mut world);
            resolve_match_launch_intent(
                match_session,
                launch_intent,
                pending_snapshot,
                menu_state,
                next_state,
            );
        }
        assert_eq!(
            world.resource::<MatchLaunchIntent>(),
            &MatchLaunchIntent::NewLocalMatch
        );
        assert_eq!(world.resource::<ShellMenuState>().panel, MenuPanel::Setup);
        assert_eq!(
            world.resource::<ShellMenuState>().context,
            MenuContext::MainMenu
        );
        assert_eq!(world.resource::<ShellMenuState>().confirmation, None);
    }

    #[test]
    fn promotion_action_applies_pending_move() {
        let mut match_session = MatchSession::start_local_match();
        let promotion_ready = match GameState::from_fen("7k/P7/8/8/8/8/8/4K3 w - - 0 1") {
            Ok(game_state) => game_state,
            Err(error) => panic!("promotion fixture FEN should parse: {error}"),
        };
        let from = match Square::from_algebraic("a7") {
            Some(square) => square,
            None => panic!("promotion fixture source square should parse"),
        };
        let to = match Square::from_algebraic("a8") {
            Some(square) => square,
            None => panic!("promotion fixture destination square should parse"),
        };
        match_session.replace_game_state(promotion_ready);
        match_session.pending_promotion_move = Some(Move::new(from, to));
        handle_promotion_action(PieceKind::Queen, &mut match_session)
            .expect("promotion action should resolve the pending move");
        assert_eq!(match_session.pending_promotion_move, None);
    }
}
