use tempfile::tempdir;

use chess_core::{DrawReason, GameOutcome, GameState, GameStatus, PieceKind, Side, Square, WinReason};
use chess_persistence::{
    GameSnapshot, SaveKind, SessionStore, SnapshotMetadata, SnapshotShellState,
};
use game_app::{
    AutomationCommand, AutomationConfirmationKind, AutomationHarness, AutomationMatchAction,
    AutomationNavigationAction, AutomationSaveAction, AutomationScreen,
};

fn sq(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn submit_move(
    harness: &mut AutomationHarness,
    from: &str,
    to: &str,
    promotion: Option<PieceKind>,
) {
    harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
            from: sq(from),
            to: sq(to),
            promotion,
        }))
        .expect("move should succeed");
}

fn start_new_match(harness: &mut AutomationHarness) {
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::StartNewMatch,
        ))
        .expect("start command should succeed");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("match loading should settle");
}

fn manual_snapshot(label: &str, fen: &str) -> GameSnapshot {
    GameSnapshot::from_parts(
        GameState::from_fen(fen).expect("fixture FEN should parse"),
        SnapshotMetadata {
            label: label.to_string(),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Manual,
            session_id: label.to_ascii_lowercase().replace(' ', "-"),
            recovery_key: None,
        },
        SnapshotShellState::default(),
    )
}

fn load_save_by_index(harness: &mut AutomationHarness, index: usize) {
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenSetup,
        ))
        .expect("setup");
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::OpenLoadList,
        ))
        .expect("load list");
    let saves = harness.snapshot().saves.manual_saves.clone();
    assert!(
        saves.len() > index,
        "expected at least {} saves, got {}",
        index + 1,
        saves.len()
    );
    let slot_id = saves[index].slot_id.clone();
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
            slot_id,
        }))
        .expect("select");
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
        .expect("load");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");
}

// ---------------------------------------------------------------------------
// Scenario 1: Complete game to checkmate
// ---------------------------------------------------------------------------

#[test]
fn complete_game_to_checkmate_via_scholars_mate() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Scholar's Mate: 1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7#
    submit_move(&mut harness, "e2", "e4", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "d1", "h5", None);
    submit_move(&mut harness, "b8", "c6", None);
    submit_move(&mut harness, "f1", "c4", None);
    submit_move(&mut harness, "g8", "f6", None);
    submit_move(&mut harness, "h5", "f7", None);

    // Extra frames for the checkmate → MatchResult screen transition.
    let snapshot = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("result transition should settle");
    assert_eq!(snapshot.screen, AutomationScreen::MatchResult);
    match snapshot.match_state.status {
        GameStatus::Finished(GameOutcome::Win {
            winner: Side::White,
            reason: WinReason::Checkmate,
        }) => {}
        ref other => panic!("expected White checkmate, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Mid-game save/load
// ---------------------------------------------------------------------------

#[test]
fn mid_game_save_load_preserves_position() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Play 4 moves: 1. e4 e5 2. Nf3 Nc6
    submit_move(&mut harness, "e2", "e4", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "g1", "f3", None);
    submit_move(&mut harness, "b8", "c6", None);

    let fen_before_save = harness.snapshot().match_state.fen.clone();

    // Save.
    harness
        .try_submit(AutomationCommand::Save(AutomationSaveAction::SaveManual {
            label: Some(String::from("mid-game test")),
        }))
        .expect("save should succeed");

    // Return to menu.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::PauseMatch,
        ))
        .expect("pause");
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::ReturnToMenu,
        ))
        .expect("return");
    harness
        .try_submit(AutomationCommand::Confirm(
            AutomationConfirmationKind::AbandonMatch,
        ))
        .expect("confirm abandon");
    harness
        .try_submit(AutomationCommand::Step { frames: 2 })
        .expect("settle");

    // Load the save.
    load_save_by_index(&mut harness, 0);

    let fen_after_load = harness.snapshot().match_state.fen.clone();
    assert_eq!(
        fen_before_save, fen_after_load,
        "FEN should match after load"
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: Promotion through automation
// ---------------------------------------------------------------------------

#[test]
fn promotion_through_automation_changes_piece_type() {
    let root = tempdir().expect("temp dir");
    let store = SessionStore::new(root.path());
    store
        .save_manual(manual_snapshot(
            "promotion-test",
            "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        ))
        .expect("save");

    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    load_save_by_index(&mut harness, 0);

    // Promote pawn to queen.
    submit_move(&mut harness, "a7", "a8", Some(PieceKind::Queen));

    let snapshot = harness.snapshot();
    assert!(
        snapshot.match_state.fen.starts_with("Q3k3"),
        "FEN should show queen on a8, got: {}",
        snapshot.match_state.fen
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: Draw claim flow
// ---------------------------------------------------------------------------

#[test]
fn draw_claim_through_automation_reaches_match_result() {
    let root = tempdir().expect("temp dir");
    let store = SessionStore::new(root.path());
    store
        .save_manual(manual_snapshot(
            "draw-claim-test",
            "4k3/8/8/8/8/8/8/4K3 w - - 100 1",
        ))
        .expect("save");

    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    load_save_by_index(&mut harness, 0);

    // Verify draw is claimable.
    let snap = harness.snapshot();
    assert!(
        snap.match_state.claimable_draw.fifty_move_rule,
        "50-move rule should be claimable"
    );

    // Claim the draw.
    harness
        .try_submit(AutomationCommand::Match(AutomationMatchAction::ClaimDraw))
        .expect("claim draw should succeed");

    let snap = harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("step");

    assert_eq!(snap.screen, AutomationScreen::MatchResult);
}

// ---------------------------------------------------------------------------
// Scenario 5: Rematch after completion
// ---------------------------------------------------------------------------

#[test]
fn rematch_after_checkmate_starts_fresh_game() {
    let root = tempdir().expect("temp dir");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    start_new_match(&mut harness);

    // Fool's Mate for quick finish.
    submit_move(&mut harness, "f2", "f3", None);
    submit_move(&mut harness, "e7", "e5", None);
    submit_move(&mut harness, "g2", "g4", None);
    submit_move(&mut harness, "d8", "h4", None);

    // Extra frames for the checkmate → MatchResult screen transition.
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("result transition");
    assert_eq!(harness.snapshot().screen, AutomationScreen::MatchResult);

    // Rematch.
    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::Rematch,
        ))
        .expect("rematch");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    let snap = harness.snapshot();
    assert_eq!(snap.screen, AutomationScreen::InMatch);
    assert!(
        snap.match_state.fen.contains("rnbqkbnr"),
        "should be starting position after rematch"
    );
}

// ---------------------------------------------------------------------------
// Scenario 6: Stalemate through full stack
// ---------------------------------------------------------------------------

#[test]
fn stalemate_detected_through_full_stack() {
    let root = tempdir().expect("temp dir");
    let store = SessionStore::new(root.path());
    store
        .save_manual(manual_snapshot(
            "stalemate-test",
            "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
        ))
        .expect("save");

    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();
    load_save_by_index(&mut harness, 0);

    let snap = harness.snapshot();
    match snap.match_state.status {
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {}
        ref other => panic!("expected stalemate, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Scenario 7: Recovery resume
// ---------------------------------------------------------------------------

#[test]
fn recovery_resume_preserves_position_after_simulated_crash() {
    let root = tempdir().expect("temp dir");
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    let game_state = GameState::from_fen(fen).expect("test FEN");
    let snapshot = GameSnapshot::from_parts(
        game_state,
        SnapshotMetadata {
            label: String::from("recovery-test"),
            created_at_utc: Some(String::from("2026-03-18T00:00:00Z")),
            updated_at_utc: None,
            notes: None,
            save_kind: SaveKind::Recovery,
            session_id: String::from("recovery"),
            recovery_key: Some(String::from("autosave")),
        },
        SnapshotShellState::default(),
    );
    SessionStore::new(root.path())
        .store_recovery(snapshot)
        .expect("save recovery");

    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    let snap = harness.snapshot();
    assert!(
        snap.menu.recovery_available,
        "recovery banner should be available"
    );

    harness
        .try_submit(AutomationCommand::Navigation(
            AutomationNavigationAction::ResumeRecovery,
        ))
        .expect("resume");
    harness
        .try_submit(AutomationCommand::Step { frames: 3 })
        .expect("settle");

    let snap = harness.snapshot();
    assert_eq!(snap.screen, AutomationScreen::InMatch);
    assert_eq!(
        snap.match_state.fen, fen,
        "FEN should match the pre-populated recovery position"
    );
}
