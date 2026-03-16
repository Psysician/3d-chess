//! Shell persistence orchestration for manual saves, interrupted-session recovery, and settings.
//! Repository I/O lives here so manual saves, interrupted-session recovery, and the shipped settings trio of startup recovery, destructive confirmations, and display mode stay behind one snapshot-based boundary. (ref: DL-002) (ref: DL-005) (ref: DL-007) (ref: DL-008)
//! Extracted helpers carry branch-heavy copy and recovery-visibility rules so the Bevy plugin remains in scope while direct tests cover the decision surface. (ref: DL-002) (ref: DL-004) (ref: DL-007)

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, Window, WindowMode};
use chess_persistence::{
    DisplayMode, RecoveryStartupPolicy, SaveKind, SavedSessionSummary, SessionStore, ShellSettings,
    SnapshotMetadata, StoreResult,
};

use super::menu::{MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState};
use super::save_load_logic;
use crate::app::AppScreenState;
use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};

pub struct SaveLoadPlugin;

type StartupRecoveryRuntime<'w> = (
    ResMut<'w, StartupRecoveryHandled>,
    ResMut<'w, RecoveryBannerState>,
    ResMut<'w, PendingLoadedSnapshot>,
    ResMut<'w, MatchLaunchIntent>,
    ResMut<'w, NextState<AppScreenState>>,
);

type SaveRequestRuntime<'w> = (
    ResMut<'w, PendingLoadedSnapshot>,
    ResMut<'w, MatchLaunchIntent>,
    ResMut<'w, SaveLoadState>,
    ResMut<'w, RecoveryBannerState>,
    ResMut<'w, ShellMenuState>,
    ResMut<'w, NextState<AppScreenState>>,
);

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct SaveRootOverride(pub Option<PathBuf>);

#[derive(Resource, Debug, Clone)]
pub struct SessionStoreResource(pub SessionStore);

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct SaveLoadState {
    pub manual_saves: Vec<SavedSessionSummary>,
    pub recovery: Option<SavedSessionSummary>,
    pub settings: ShellSettings,
    pub last_message: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum SaveLoadRequest {
    RefreshIndex,
    SaveManual {
        label: String,
        slot_id: Option<String>,
    },
    LoadManual {
        slot_id: String,
    },
    DeleteManual {
        slot_id: String,
    },
    ResumeRecovery,
    ClearRecovery,
    AbandonMatchAndReturnToMenu,
    PersistSettings,
}

#[derive(Resource, Default)]
struct StartupRecoveryHandled(bool);

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveRootOverride>()
            .init_resource::<SaveLoadState>()
            .init_resource::<StartupRecoveryHandled>()
            .add_message::<SaveLoadRequest>()
            .add_systems(Startup, setup_store)
            .add_systems(
                Update,
                (
                    maybe_resume_recovery_on_startup,
                    apply_display_mode_setting,
                    handle_save_load_requests,
                    autosave_active_match,
                ),
            )
            .add_systems(OnEnter(AppScreenState::MatchResult), clear_result_recovery);
    }
}

fn setup_store(
    mut commands: Commands,
    root_override: Res<SaveRootOverride>,
    mut save_state: ResMut<SaveLoadState>,
    mut recovery_banner: ResMut<RecoveryBannerState>,
) {
    let (store, startup_error) = match root_override.0.clone() {
        Some(root) => (SessionStore::new(root), None),
        None => resolve_session_store(None, SessionStore::runtime()),
    };
    let store_resource = SessionStoreResource(store.clone());
    // Startup preloads the save index and recovery banner from the repository so the main menu reflects persisted shell state immediately. (ref: DL-003) (ref: DL-008)
    save_state.last_error = save_load_logic::combine_persistence_errors([
        startup_error,
        refresh_store_index_from_resource(&store_resource, &mut save_state, &mut recovery_banner),
    ]);
    commands.insert_resource(SessionStoreResource(store));
}

fn maybe_resume_recovery_on_startup(
    state: Res<State<AppScreenState>>,
    store: Res<SessionStoreResource>,
    save_state: Res<SaveLoadState>,
    startup_runtime: StartupRecoveryRuntime<'_>,
) {
    let (mut handled, mut recovery_banner, mut pending_snapshot, mut launch_intent, mut next_state) =
        startup_runtime;

    if handled.0 || *state.get() != AppScreenState::MainMenu {
        return;
    }

    handled.0 = true;
    match save_state.settings.recovery_policy {
        RecoveryStartupPolicy::Resume => {
            if let Ok(Some(snapshot)) = store.0.load_recovery() {
                pending_snapshot.0 = Some(snapshot);
                *launch_intent = MatchLaunchIntent::ResumeRecovery;
                next_state.set(AppScreenState::MatchLoading);
            }
        }
        RecoveryStartupPolicy::Ignore => {
            save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
        }
        RecoveryStartupPolicy::Ask => {}
    }
}

fn apply_display_mode_setting(
    save_state: Res<SaveLoadState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !save_state.is_changed() {
        return;
    }

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.mode = match save_state.settings.display_mode {
        DisplayMode::Windowed => WindowMode::Windowed,
        DisplayMode::Fullscreen => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
    };
}

fn handle_save_load_requests(
    mut requests: MessageReader<SaveLoadRequest>,
    store: Res<SessionStoreResource>,
    match_session: Res<MatchSession>,
    save_runtime: SaveRequestRuntime<'_>,
) {
    let (
        mut pending_snapshot,
        mut launch_intent,
        mut save_state,
        mut recovery_banner,
        mut menu_state,
        mut next_state,
    ) = save_runtime;

    for request in requests.read() {
        match request {
            SaveLoadRequest::RefreshIndex => {
                save_state.last_error = refresh_store_index_from_resource(
                    &store,
                    &mut save_state,
                    &mut recovery_banner,
                );
            }
            SaveLoadRequest::SaveManual { label, slot_id } => {
                let snapshot = match_session.to_snapshot(SnapshotMetadata {
                    label: label.clone(),
                    created_at_utc: None,
                    updated_at_utc: None,
                    notes: Some(String::from("Manual save")),
                    save_kind: SaveKind::Manual,
                    session_id: slot_id.clone().unwrap_or_default(),
                    recovery_key: None,
                });

                match store.0.save_manual(snapshot) {
                    Ok(summary) => {
                        save_state.last_error = None;
                        save_state.last_message =
                            Some(save_load_logic::manual_save_message(&summary));
                        menu_state.selected_save = Some(summary.slot_id.clone());
                        save_state.last_error = refresh_store_index_from_resource(
                            &store,
                            &mut save_state,
                            &mut recovery_banner,
                        );
                    }
                    Err(_) => {
                        save_state.last_error =
                            Some(String::from("Unable to write the selected save slot."));
                    }
                }
            }
            SaveLoadRequest::LoadManual { slot_id } => match store.0.load_manual(slot_id) {
                Ok(snapshot) => {
                    pending_snapshot.0 = Some(snapshot);
                    *launch_intent = MatchLaunchIntent::LoadManual;
                    save_state.last_error = None;
                    save_state.last_message = Some(format!("Loading save {slot_id}."));
                    next_state.set(AppScreenState::MatchLoading);
                }
                Err(_) => {
                    save_state.last_error = Some(String::from("Unable to load the selected save."));
                }
            },
            SaveLoadRequest::DeleteManual { slot_id } => match store.0.delete_manual(slot_id) {
                Ok(()) => {
                    save_state.last_error = None;
                    save_state.last_message = Some(save_load_logic::deleted_save_message(slot_id));
                    if menu_state.selected_save.as_deref() == Some(slot_id.as_str()) {
                        menu_state.selected_save = None;
                    }
                    save_state.last_error = refresh_store_index_from_resource(
                        &store,
                        &mut save_state,
                        &mut recovery_banner,
                    );
                }
                Err(_) => {
                    save_state.last_error =
                        Some(String::from("Unable to delete the selected save."));
                }
            },
            SaveLoadRequest::ResumeRecovery => match store.0.load_recovery() {
                Ok(Some(snapshot)) => {
                    pending_snapshot.0 = Some(snapshot);
                    *launch_intent = MatchLaunchIntent::ResumeRecovery;
                    save_state.last_error = None;
                    save_state.last_message = Some(String::from("Resuming interrupted session."));
                    next_state.set(AppScreenState::MatchLoading);
                }
                Ok(None) => {
                    save_state.last_error =
                        Some(String::from("No interrupted session is available."));
                }
                Err(_) => {
                    save_state.last_error =
                        Some(String::from("Unable to resume the interrupted session."));
                }
            },
            SaveLoadRequest::ClearRecovery => match store.0.clear_recovery() {
                Ok(()) => {
                    clear_cached_recovery(&mut save_state, &mut recovery_banner);
                }
                Err(_) => {
                    save_state.last_error = Some(String::from(
                        "Unable to clear interrupted-session recovery.",
                    ));
                }
            },
            SaveLoadRequest::AbandonMatchAndReturnToMenu => match store.0.clear_recovery() {
                Ok(()) => {
                    clear_cached_recovery(&mut save_state, &mut recovery_banner);
                    save_state.last_error = None;
                    save_state.last_message = Some(String::from("Returned to the main menu."));
                    menu_state.panel = MenuPanel::Home;
                    menu_state.context = MenuContext::MainMenu;
                    menu_state.confirmation = None;
                    next_state.set(AppScreenState::MainMenu);
                }
                Err(_) => {
                    save_state.last_error = Some(String::from(
                        "Unable to clear interrupted-session recovery.",
                    ));
                }
            },
            SaveLoadRequest::PersistSettings => match store.0.save_settings(&save_state.settings) {
                Ok(()) => {
                    save_state.last_error = None;
                    save_state.last_message = Some(String::from("Saved shell settings."));
                    save_load_logic::sync_cached_recovery_visibility(
                        &save_state,
                        &mut recovery_banner,
                    );
                }
                Err(_) => {
                    save_state.last_error = Some(String::from("Unable to save shell settings."));
                    save_load_logic::sync_cached_recovery_visibility(
                        &save_state,
                        &mut recovery_banner,
                    );
                }
            },
        }
    }
}

fn autosave_active_match(
    state: Res<State<AppScreenState>>,
    store: Res<SessionStoreResource>,
    mut match_session: ResMut<MatchSession>,
    mut save_state: ResMut<SaveLoadState>,
    mut recovery_banner: ResMut<RecoveryBannerState>,
) {
    if *state.get() != AppScreenState::InMatch || !match_session.is_changed() {
        return;
    }

    let mut snapshot = match_session.to_snapshot(SnapshotMetadata {
        label: String::from("Interrupted Session"),
        created_at_utc: None,
        updated_at_utc: None,
        notes: Some(String::from("Automatic recovery snapshot")),
        save_kind: SaveKind::Recovery,
        session_id: String::new(),
        recovery_key: Some(String::from("autosave")),
    });
    // The on-disk recovery record represents the last persisted state, so the dirty flag clears before write. (ref: DL-003)
    snapshot.shell_state.dirty_recovery = false;

    match store.0.store_recovery(snapshot) {
        Ok(summary) => {
            match_session.mark_recovery_persisted();
            set_cached_recovery(Some(summary), &mut save_state, &mut recovery_banner);
        }
        Err(_) => {
            save_state.last_error = Some(String::from(
                "Unable to refresh interrupted-session recovery.",
            ));
        }
    }
}

fn clear_result_recovery(
    store: Res<SessionStoreResource>,
    mut save_state: ResMut<SaveLoadState>,
    mut recovery_banner: ResMut<RecoveryBannerState>,
) {
    clear_result_recovery_cache(&store, &mut save_state, &mut recovery_banner);
}

fn clear_result_recovery_cache(
    store: &SessionStoreResource,
    save_state: &mut SaveLoadState,
    recovery_banner: &mut RecoveryBannerState,
) {
    match store.0.clear_recovery() {
        Ok(()) => clear_cached_recovery(save_state, recovery_banner),
        Err(_) => {
            save_state.last_error = Some(String::from(
                "Unable to clear interrupted-session recovery.",
            ));
            save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
        }
    }
}

fn refresh_store_index_from_resource(
    store: &SessionStoreResource,
    save_state: &mut SaveLoadState,
    recovery_banner: &mut RecoveryBannerState,
) -> Option<String> {
    let mut errors = Vec::new();

    match store.0.list_manual_saves() {
        Ok(manual_saves) => {
            save_state.manual_saves = manual_saves;
        }
        Err(error) => errors.push(format!("Unable to refresh save index: {error}.")),
    }

    match store.0.load_settings() {
        Ok(settings) => {
            save_state.settings = settings;
        }
        Err(error) => errors.push(format!("Unable to load shell settings: {error}.")),
    }

    match store.0.load_recovery() {
        Ok(recovery) => {
            set_cached_recovery(
                // Banner state recomputes from storage here so stale dirty flags never leak back into the shell. (ref: DL-003)
                recovery.map(|snapshot| SavedSessionSummary::from_snapshot(&snapshot)),
                save_state,
                recovery_banner,
            );
        }
        Err(error) => errors.push(format!(
            "Unable to inspect interrupted-session recovery: {error}."
        )),
    }

    save_load_logic::combine_persistence_errors(errors.into_iter().map(Some))
}

fn resolve_session_store(
    root_override: Option<PathBuf>,
    runtime_store: StoreResult<SessionStore>,
) -> (SessionStore, Option<String>) {
    if let Some(root) = root_override {
        return (SessionStore::new(root), None);
    }

    match runtime_store {
        Ok(store) => (store, None),
        Err(error) => {
            let fallback_root = std::env::temp_dir().join("3d-chess");
            (
                SessionStore::new(fallback_root.clone()),
                Some(format!(
                    "Save storage is using fallback root {} because the default app-data directory is unavailable: {error}.",
                    fallback_root.display()
                )),
            )
        }
    }
}

fn set_cached_recovery(
    recovery: Option<SavedSessionSummary>,
    save_state: &mut SaveLoadState,
    recovery_banner: &mut RecoveryBannerState,
) {
    save_state.recovery = recovery;
    save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
}

fn clear_cached_recovery(
    save_state: &mut SaveLoadState,
    recovery_banner: &mut RecoveryBannerState,
) {
    save_state.recovery = None;
    save_load_logic::hide_recovery_banner(recovery_banner);
}

#[cfg(test)]
mod tests {
    use super::*;

    use chess_persistence::ShellSettings;
    use tempfile::tempdir;

    #[test]
    fn resolve_session_store_surfaces_runtime_fallback_errors() {
        let fallback_root = std::env::temp_dir().join("3d-chess");
        let (store, error) =
            resolve_session_store(None, Err(chess_persistence::StoreError::MissingPlatformDir));

        assert_eq!(store.root(), fallback_root.as_path());
        assert!(
            error
                .expect("runtime fallback should surface an error")
                .contains("fallback root")
        );
    }

    #[test]
    fn refresh_store_index_reports_corrupt_save_data_without_resetting_existing_index() {
        let root = tempdir().expect("temporary directory should be created");
        let saves_dir = root.path().join("saves");
        std::fs::create_dir_all(&saves_dir).expect("save directory should exist");
        std::fs::write(saves_dir.join("broken.json"), "{not-json")
            .expect("broken save fixture should be written");

        let store = SessionStoreResource(SessionStore::new(root.path()));
        let mut save_state = SaveLoadState {
            manual_saves: vec![SavedSessionSummary {
                slot_id: String::from("existing"),
                label: String::from("Existing"),
                created_at_utc: None,
                save_kind: SaveKind::Manual,
            }],
            ..Default::default()
        };
        let mut recovery_banner = RecoveryBannerState::default();

        let error =
            refresh_store_index_from_resource(&store, &mut save_state, &mut recovery_banner)
                .expect("corrupt index should surface an error");

        assert!(error.contains("Unable to refresh save index"));
        assert_eq!(save_state.manual_saves.len(), 1);
        assert_eq!(save_state.manual_saves[0].slot_id, "existing");
    }

    #[test]
    fn refresh_store_index_keeps_ignored_recovery_hidden() {
        let root = tempdir().expect("temporary directory should be created");
        let store = SessionStore::new(root.path());
        store
            .store_recovery(chess_persistence::GameSnapshot::new(
                chess_core::GameState::starting_position(),
                SnapshotMetadata {
                    label: String::from("Ignored Recovery"),
                    created_at_utc: None,
                    updated_at_utc: None,
                    notes: None,
                    save_kind: SaveKind::Recovery,
                    session_id: String::from("recovery"),
                    recovery_key: Some(String::from("autosave")),
                },
            ))
            .expect("recovery snapshot should be written");
        store
            .save_settings(&ShellSettings {
                recovery_policy: RecoveryStartupPolicy::Ignore,
                ..ShellSettings::default()
            })
            .expect("ignore policy should be written");

        let store = SessionStoreResource(store);
        let mut save_state = SaveLoadState::default();
        let mut recovery_banner = RecoveryBannerState {
            available: true,
            dirty: false,
            label: Some(String::from("stale")),
        };

        let error =
            refresh_store_index_from_resource(&store, &mut save_state, &mut recovery_banner);

        assert_eq!(error, None);
        assert_eq!(
            save_state
                .recovery
                .as_ref()
                .map(|summary| summary.label.as_str()),
            Some("Ignored Recovery")
        );
        assert!(!recovery_banner.available);
        assert_eq!(recovery_banner.label, None);
    }

    #[test]
    fn cached_recovery_visibility_reappears_when_ignore_is_disabled() {
        let mut save_state = SaveLoadState {
            recovery: Some(SavedSessionSummary {
                slot_id: String::from("recovery"),
                label: String::from("Recovery Fixture"),
                created_at_utc: None,
                save_kind: SaveKind::Recovery,
            }),
            settings: ShellSettings {
                recovery_policy: RecoveryStartupPolicy::Ignore,
                ..ShellSettings::default()
            },
            ..Default::default()
        };
        let mut recovery_banner = RecoveryBannerState::default();

        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
        assert!(!recovery_banner.available);

        save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);

        assert!(recovery_banner.available);
        assert_eq!(recovery_banner.label.as_deref(), Some("Recovery Fixture"));
    }

    #[test]
    fn clear_result_recovery_keeps_cached_recovery_when_store_clear_fails() {
        let root = tempdir().expect("temporary directory should be created");
        std::fs::create_dir_all(root.path().join("recovery").join("current.json"))
            .expect("recovery fixture directory should be created");

        let store = SessionStoreResource(SessionStore::new(root.path()));
        let mut save_state = SaveLoadState {
            recovery: Some(SavedSessionSummary {
                slot_id: String::from("recovery"),
                label: String::from("Interrupted Session"),
                created_at_utc: None,
                save_kind: SaveKind::Recovery,
            }),
            settings: ShellSettings::default(),
            ..Default::default()
        };
        let mut recovery_banner = RecoveryBannerState::default();

        save_load_logic::sync_cached_recovery_visibility(&save_state, &mut recovery_banner);
        clear_result_recovery_cache(&store, &mut save_state, &mut recovery_banner);

        assert_eq!(
            save_state.last_error.as_deref(),
            Some("Unable to clear interrupted-session recovery.")
        );
        assert_eq!(
            save_state
                .recovery
                .as_ref()
                .map(|summary| summary.slot_id.as_str()),
            Some("recovery")
        );
        assert!(recovery_banner.available);
        assert_eq!(
            recovery_banner.label.as_deref(),
            Some("Interrupted Session")
        );
    }
}
