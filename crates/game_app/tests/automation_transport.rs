// Transport contract coverage locks the request and response envelope without
// depending on a native window. (refs: DL-005, DL-006)

#![cfg(feature = "automation-transport")]

use std::io::Cursor;

use tempfile::tempdir;

use game_app::{
    automation_transport::{
        AutomationRequest, AutomationResponse, AutomationTransportErrorCode, run_stdio_session,
    },
    AutomationCommand, AutomationHarness, AutomationNavigationAction, AutomationSaveAction,
    AutomationScreen,
};

fn decode(output: Vec<u8>) -> Vec<AutomationResponse> {
    String::from_utf8(output)
        .expect("transport output should stay utf8")
        .lines()
        .map(|line| {
            serde_json::from_str::<AutomationResponse>(line)
                .expect("each response line should parse")
        })
        .collect()
}

#[test]
fn stdio_roundtrips_snapshots_for_representative_commands() {
    let root = tempdir().expect("temporary directory should be created");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    let requests = [
        AutomationRequest {
            command: AutomationCommand::Navigation(AutomationNavigationAction::OpenSetup),
        },
        AutomationRequest {
            command: AutomationCommand::Navigation(AutomationNavigationAction::StartNewMatch),
        },
        AutomationRequest {
            command: AutomationCommand::Step { frames: 3 },
        },
    ];
    let mut input = String::new();
    for request in requests {
        input.push_str(&serde_json::to_string(&request).expect("request should serialize"));
        input.push('\n');
    }

    let mut output = Vec::new();
    run_stdio_session(Cursor::new(input.into_bytes()), &mut output, &mut harness)
        .expect("stdio transport should succeed");

    let responses = decode(output);
    assert_eq!(responses.len(), 3);
    assert!(responses.iter().all(|response| response.error.is_none()));
    assert_eq!(
        responses.last().and_then(|response| response.snapshot.as_ref()).map(|snapshot| snapshot.screen),
        Some(AutomationScreen::InMatch)
    );
}

#[test]
fn stdio_returns_structured_errors_for_invalid_json_and_missing_save_selection() {
    let root = tempdir().expect("temporary directory should be created");
    let mut harness =
        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
    harness.boot_to_main_menu();

    let valid = serde_json::to_string(&AutomationRequest {
        command: AutomationCommand::Save(AutomationSaveAction::LoadSelected),
    })
    .expect("request should serialize");
    let input = format!("{{not json}}\n{valid}\n");

    let mut output = Vec::new();
    run_stdio_session(Cursor::new(input.into_bytes()), &mut output, &mut harness)
        .expect("transport should still emit structured errors");

    let responses = decode(output);
    assert_eq!(responses[0].error.as_ref().map(|error| error.code), Some(AutomationTransportErrorCode::InvalidRequest));
    assert_eq!(responses[1].error.as_ref().map(|error| error.code), Some(AutomationTransportErrorCode::CommandRejected));
}
