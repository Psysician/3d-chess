#![cfg(all(feature = "automation-transport", unix))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use chess_core::{GameState, GameStatus, Move, PieceKind, Side, Square};
use chess_persistence::{
    GameSnapshot, SaveKind, SessionStore, SnapshotMetadata, SnapshotShellState,
};
use game_app::automation_transport::{AutomationRequest, AutomationResponse};
use game_app::{
    AutomationCommand, AutomationMatchAction, AutomationNavigationAction, AutomationSaveAction,
    AutomationScreen, AutomationSnapshot,
};
use tempfile::{TempDir, tempdir};

fn square(name: &str) -> Square {
    Square::from_algebraic(name).expect("test square must be valid")
}

fn apply_moves(mut state: GameState, moves: &[Move]) -> GameState {
    for candidate in moves {
        state = state
            .apply_move(*candidate)
            .expect("fixture move should be legal");
    }
    state
}

fn manual_snapshot(label: &str, fen: &str) -> GameSnapshot {
    GameSnapshot::from_parts(
        GameState::from_fen(fen).expect("fixture FEN should parse"),
        SnapshotMetadata {
            label: label.to_string(),
            created_at_utc: Some(String::from("2026-03-17T00:00:00Z")),
            updated_at_utc: None,
            notes: Some(String::from("real-world round fixture")),
            save_kind: SaveKind::Manual,
            session_id: String::new(),
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
            notes: Some(String::from("real-world round recovery fixture")),
            save_kind: SaveKind::Recovery,
            session_id: String::from("recovery"),
            recovery_key: Some(String::from("autosave")),
        },
        SnapshotShellState::default(),
    )
}

struct AgentRound {
    _data_home: TempDir,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl AgentRound {
    fn new() -> Self {
        Self::spawn(tempdir().expect("temporary data root should be created"))
    }

    fn with_store_setup(setup: impl FnOnce(&SessionStore)) -> Self {
        let data_home = tempdir().expect("temporary data root should be created");
        let store = SessionStore::new(runtime_store_root(data_home.path()));
        setup(&store);
        Self::spawn(data_home)
    }

    fn spawn(data_home: TempDir) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_game_app_agent"))
            .env("XDG_DATA_HOME", data_home.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("game_app_agent should spawn");

        let stdin = child.stdin.take().expect("child stdin should be piped");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout should be piped"));

        Self {
            _data_home: data_home,
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, command: AutomationCommand) -> AutomationResponse {
        let request = AutomationRequest { command };
        let payload = serde_json::to_string(&request).expect("request should serialize");
        writeln!(self.stdin, "{payload}").expect("request should be written");
        self.stdin.flush().expect("stdin should flush");

        let mut line = String::new();
        let bytes = self
            .stdout
            .read_line(&mut line)
            .expect("agent response should be readable");
        assert!(bytes > 0, "agent closed stdout before replying");

        serde_json::from_str::<AutomationResponse>(line.trim_end())
            .expect("response should deserialize")
    }

    fn send_ok(&mut self, command: AutomationCommand) -> AutomationSnapshot {
        let response = self.send(command);
        assert!(
            response.error.is_none(),
            "agent command returned an error: {:?}",
            response.error
        );
        response
            .snapshot
            .expect("successful responses should include a snapshot")
    }
}

impl Drop for AgentRound {
    fn drop(&mut self) {
        if self
            .child
            .try_wait()
            .expect("child wait should succeed")
            .is_none()
        {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn runtime_store_root(data_home: &Path) -> PathBuf {
    data_home.join("3d-chess")
}

fn assert_match_result(snapshot: &AutomationSnapshot, expected: &GameState) {
    assert_eq!(snapshot.screen, AutomationScreen::MatchResult);
    assert_eq!(snapshot.match_state.fen, expected.to_fen());
    assert_eq!(snapshot.match_state.status, expected.status());
    assert!(snapshot.saves.recovery.is_none());
}

#[test]
fn round_one_fools_mate_reaches_result_screen() {
    let mut round = AgentRound::new();
    let mating_line = [
        Move::new(square("f2"), square("f3")),
        Move::new(square("e7"), square("e5")),
        Move::new(square("g2"), square("g4")),
        Move::new(square("d8"), square("h4")),
    ];
    let expected = apply_moves(GameState::starting_position(), &mating_line);

    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::StartNewMatch,
    ));
    let snapshot = round.send_ok(AutomationCommand::Step { frames: 3 });
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);

    for candidate in mating_line {
        round.send_ok(AutomationCommand::Match(
            AutomationMatchAction::SubmitMove {
                from: candidate.from(),
                to: candidate.to(),
                promotion: candidate.promotion(),
            },
        ));
    }

    let snapshot = round.send_ok(AutomationCommand::Step { frames: 2 });
    assert_match_result(&snapshot, &expected);
    assert_eq!(
        snapshot.match_state.status,
        GameStatus::Finished(chess_core::GameOutcome::Win {
            winner: Side::Black,
            reason: chess_core::WinReason::Checkmate,
        })
    );
}

#[test]
fn round_two_scholars_mate_reaches_result_screen() {
    let mut round = AgentRound::new();
    let mating_line = [
        Move::new(square("e2"), square("e4")),
        Move::new(square("e7"), square("e5")),
        Move::new(square("d1"), square("h5")),
        Move::new(square("b8"), square("c6")),
        Move::new(square("f1"), square("c4")),
        Move::new(square("g8"), square("f6")),
        Move::new(square("h5"), square("f7")),
    ];
    let expected = apply_moves(GameState::starting_position(), &mating_line);

    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::StartNewMatch,
    ));
    round.send_ok(AutomationCommand::Step { frames: 3 });

    for candidate in mating_line {
        round.send_ok(AutomationCommand::Match(
            AutomationMatchAction::SubmitMove {
                from: candidate.from(),
                to: candidate.to(),
                promotion: candidate.promotion(),
            },
        ));
    }

    let snapshot = round.send_ok(AutomationCommand::Step { frames: 2 });
    assert_match_result(&snapshot, &expected);
    assert_eq!(
        snapshot.match_state.status,
        GameStatus::Finished(chess_core::GameOutcome::Win {
            winner: Side::White,
            reason: chess_core::WinReason::Checkmate,
        })
    );
}

#[test]
fn round_three_save_load_then_finish_from_restored_position() {
    let mut round = AgentRound::new();
    let restored_line = [
        Move::new(square("f2"), square("f3")),
        Move::new(square("e7"), square("e5")),
        Move::new(square("g2"), square("g4")),
        Move::new(square("d8"), square("h4")),
    ];
    let expected = apply_moves(GameState::starting_position(), &restored_line);

    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::StartNewMatch,
    ));
    round.send_ok(AutomationCommand::Step { frames: 3 });
    let snapshot = round.send_ok(AutomationCommand::Save(AutomationSaveAction::SaveManual {
        label: Some(String::from("Start Save")),
    }));
    let snapshot = if snapshot.saves.manual_saves.is_empty() {
        round.send_ok(AutomationCommand::Step { frames: 2 })
    } else {
        snapshot
    };
    assert_eq!(snapshot.saves.manual_saves.len(), 1);
    let slot_id = snapshot.saves.manual_saves[0].slot_id.clone();

    round.send_ok(AutomationCommand::Match(
        AutomationMatchAction::SubmitMove {
            from: square("e2"),
            to: square("e4"),
            promotion: None,
        },
    ));
    round.send_ok(AutomationCommand::Match(
        AutomationMatchAction::SubmitMove {
            from: square("e7"),
            to: square("e5"),
            promotion: None,
        },
    ));
    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::PauseMatch,
    ));
    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::OpenLoadList,
    ));
    round.send_ok(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
        slot_id,
    }));
    round.send_ok(AutomationCommand::Save(AutomationSaveAction::LoadSelected));
    let snapshot = round.send_ok(AutomationCommand::Step { frames: 3 });
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
    assert_eq!(
        snapshot.match_state.fen,
        GameState::starting_position().to_fen()
    );

    for candidate in restored_line {
        round.send_ok(AutomationCommand::Match(
            AutomationMatchAction::SubmitMove {
                from: candidate.from(),
                to: candidate.to(),
                promotion: candidate.promotion(),
            },
        ));
    }

    let snapshot = round.send_ok(AutomationCommand::Step { frames: 2 });
    assert_match_result(&snapshot, &expected);
}

#[test]
fn round_four_resume_recovery_then_finish_the_game() {
    let recovery_fen = "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2";
    let mut round = AgentRound::with_store_setup(|store| {
        store
            .store_recovery(recovery_snapshot("Recovery Round", recovery_fen))
            .expect("recovery fixture should be written");
    });
    let expected = GameState::from_fen(recovery_fen)
        .expect("recovery FEN should parse")
        .apply_move(Move::new(square("d8"), square("h4")))
        .expect("recovery mate-in-one should be legal");

    let snapshot = round.send_ok(AutomationCommand::Snapshot);
    assert!(snapshot.menu.recovery_available);

    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::ResumeRecovery,
    ));
    let snapshot = round.send_ok(AutomationCommand::Step { frames: 3 });
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
    round.send_ok(AutomationCommand::Match(
        AutomationMatchAction::SubmitMove {
            from: square("d8"),
            to: square("h4"),
            promotion: None,
        },
    ));

    let snapshot = round.send_ok(AutomationCommand::Step { frames: 2 });
    assert_match_result(&snapshot, &expected);
}

#[test]
fn round_five_load_promotion_fixture_and_promote_to_mate() {
    let promotion_fen = "7k/5KP1/8/8/8/8/8/8 w - - 0 1";
    let mut round = AgentRound::with_store_setup(|store| {
        store
            .save_manual(manual_snapshot("Promotion Mate", promotion_fen))
            .expect("promotion fixture should be written");
    });
    let expected = GameState::from_fen(promotion_fen)
        .expect("promotion FEN should parse")
        .apply_move(Move::with_promotion(
            square("g7"),
            square("g8"),
            PieceKind::Queen,
        ))
        .expect("promotion mate should be legal");
    assert!(expected.status().is_finished());

    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::OpenSetup,
    ));
    round.send_ok(AutomationCommand::Navigation(
        AutomationNavigationAction::OpenLoadList,
    ));
    round.send_ok(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
        slot_id: String::from("promotion-mate"),
    }));
    round.send_ok(AutomationCommand::Save(AutomationSaveAction::LoadSelected));
    let snapshot = round.send_ok(AutomationCommand::Step { frames: 3 });
    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
    assert_eq!(snapshot.match_state.fen, promotion_fen);

    let snapshot = round.send_ok(AutomationCommand::Match(
        AutomationMatchAction::SubmitMove {
            from: square("g7"),
            to: square("g8"),
            promotion: None,
        },
    ));
    assert_eq!(
        snapshot.match_state.pending_promotion,
        Some(Move::new(square("g7"), square("g8")))
    );

    round.send_ok(AutomationCommand::Match(
        AutomationMatchAction::ChoosePromotion {
            piece: PieceKind::Queen,
        },
    ));
    let snapshot = round.send_ok(AutomationCommand::Step { frames: 2 });
    assert_match_result(&snapshot, &expected);
}
