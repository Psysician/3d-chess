use tempfile::tempdir;

use chess_core::{GameState, Move, PieceKind, Square};
use chess_persistence::{GameSnapshot, SaveKind, SessionStore, SnapshotMetadata, SnapshotShellState};
use game_app::{
    AutomationCommand, AutomationConfirmationKind, AutomationHarness,
    AutomationMatchAction, AutomationNavigationAction, AutomationSaveAction,
    AutomationScreen,
};

fn square(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn manual_snapshot(label: &str, fen: &str) -> GameSnapshot {
    GameSnapshot::from_parts(
        GameState::from_fen(fen).expect("fixture FEN should parse"),
        SnapshotMetadata {
            label: label.to_string(),
            created_at_utc: Some(String::from("2026-03-17T00:00:00Z")),
            updated_at_utc: None,
            notes: Some(String::from("automation fixture")),
            save_kind: SaveKind::Manual,
            session_id: label.to_ascii_lowercase().replace(' ', "-"),
            recovery_key: None,
        },
        SnapshotShellState::default(),
    )
}

fn recovery_snapshot(label: &str, fen: &str) -> GameSnapshot {
    GameSnapshot::from_parts(
        GameState::from_fen(fen).expect("fixture FEN should parse"),
        SnapshotMetadata {
            label: label.to_string(),
            created_at_utc: Some(String::from("2026-03-17T00:00:00Z")),
            updated_at_utc: None,
            notes: Some(String::from("recovery fixture")),
            save_kind: SaveKind::Recovery,
            session_id: String::from("recovery"),
            recovery_key: Some(String::from("autosave")),
        },
        SnapshotShellState::default(),
    )
}

#[test]
fn automation_commands_cover_start_move_save_rematch_and_return_to_menu() {
    let root = tempdir().expect("temporary directory should be created");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::StartNewMatch,
        ))
        .expect("start command should route through automation");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("match loading should settle");
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);

    let snapshot = harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
            from: square("e2"),
            to: square("e4"),
            promotion: None,
        }))
        .expect("semantic move should reuse the same legality path");
    assert_eq!(snapshot.match_state.last_move, Some(Move::new(square("e2"), square("e4"))));

    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SaveManual {
            label: Some(String::from("Automation Save")),
        }))
        .expect("manual save should be routable");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 2 })
        .expect("save refresh should settle");
    assert_eq!(snapshot.saves.manual_saves.len(), 1);

    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id: snapshot.saves.manual_saves[0].slot_id.clone(),
        }))
        .expect("slot selection should be routable");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load should be routable");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("loaded match should settle");
    assert_eq!(snapshot.match_state.last_move, Some(Move::new(square("e2"), square("e4"))));

    harness
        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::Rematch))
        .expect("rematch should be routable");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("rematch should settle");
    assert_eq!(snapshot.match_state.last_move, None);

    harness
        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::PauseMatch))
        .expect("pause should be routable");
    harness
        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::ReturnToMenu))
        .expect("return to menu should be routable");
    harness
        .try_submit(AutomationCommand::Confirm(AutomationConfirmationKind::AbandonMatch))
        .expect("abandon confirmation should be routable");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 2 })
        .expect("menu transition should settle");
    assert_eq!(snapshot.screen, AutomationScreen::MainMenu);
}

#[test]
fn automation_commands_cover_load_and_promotion_choice() {
    let root = tempdir().expect("temporary directory should be created");
    let summary = SessionStore::new(root.path())
        .save_manual(manual_snapshot("Promotion Fixture", "7k/P7/8/8/8/8/8/4K3 w - - 0 1"))
        .expect("fixture save should succeed");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    harness
        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::OpenSetup))
        .expect("setup should be routable");
    harness
        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::OpenLoadList))
        .expect("load list should be routable");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id: summary.slot_id.clone(),
        }))
        .expect("slot selection should be routable");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load should be routable");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("loaded promotion fixture should settle");

    let snapshot = harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
            from: square("a7"),
            to: square("a8"),
            promotion: None,
        }))
        .expect("promotion setup move should be routable");
    assert_eq!(snapshot.match_state.pending_promotion, Some(Move::new(square("a7"), square("a8"))));

    let snapshot = harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::ChoosePromotion {
            piece: PieceKind::Queen,
        }))
        .expect("promotion choice should be routable");
    assert_eq!(
        snapshot.match_state.last_move,
        Some(Move::with_promotion(square("a7"), square("a8"), PieceKind::Queen))
    );
}

#[test]
fn automation_commands_cover_recovery_resume() {
    let root = tempdir().expect("temporary directory should be created");
    SessionStore::new(root.path())
        .store_recovery(recovery_snapshot("Recovery Fixture", "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1"))
        .expect("recovery fixture should succeed");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    let snapshot = harness.snapshot();
    assert!(snapshot.menu.recovery_available);
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::ResumeRecovery,
        ))
        .expect("recovery resume should be routable");
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("recovery resume should settle");
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
    assert_eq!(snapshot.match_state.fen, "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");
}
