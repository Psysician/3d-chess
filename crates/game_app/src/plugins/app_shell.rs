use bevy::prelude::*;

use crate::app::AppScreenState;
use crate::style::ShellTheme;

pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (configure_ambient_light, spawn_shell_camera))
            .add_systems(OnEnter(AppScreenState::Boot), advance_to_main_menu)
            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_shell_ui)
            .add_systems(
                Update,
                orbit_camera.run_if(in_state(AppScreenState::MainMenu)),
            );
    }
}

#[derive(Component)]
struct MainMenuUi;

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
                        Text::new("Rust workspace + Bevy shell baseline"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                    panel.spawn((
                        Text::new(
                            "M0 is locking the camera, materials, lighting, crate boundaries, \
and desktop-native delivery path before gameplay systems land.",
                        ),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.ui_text),
                    ));
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
                        "Procedural board shell only in M0",
                        "Mouse + keyboard gameplay arrives in M2",
                        "Stockfish/UCI boundary reserved for M4",
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
