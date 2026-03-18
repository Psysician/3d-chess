// Integration coverage for the headless harness contract.
// These assertions keep the automation seam aligned with the shipped shell
// startup path. (refs: DL-004, DL-006)

use tempfile::tempdir;

use bevy::prelude::*;
use game_app::{
    AppScreenState, AutomationHarness, AutomationMenuPanel, AutomationScreen, SaveRootOverride,
};

#[test]
fn harness_boots_to_main_menu_and_reads_initial_snapshot() {
    let root = tempdir().expect("temporary directory should be created");
    let mut harness = AutomationHarness::new(Some(root.path().to_path_buf()));

    harness.boot_to_main_menu();
    let snapshot = harness.snapshot();

    assert_eq!(snapshot.screen, AutomationScreen::MainMenu);
    assert_eq!(snapshot.menu.panel, AutomationMenuPanel::Home);
    assert!(snapshot.saves.manual_saves.is_empty());
    assert_eq!(snapshot.menu.selected_save, None);
}

#[test]
fn windowed_builder_keeps_default_shell_startup_contract() {
    let harness = AutomationHarness::new(None);

    assert_eq!(
        harness
            .app()
            .world()
            .resource::<State<AppScreenState>>()
            .get(),
        &AppScreenState::Boot
    );
    assert_eq!(
        harness.app().world().resource::<SaveRootOverride>(),
        &SaveRootOverride::default()
    );
}
