// JSON Lines transport frames the shared automation contract while gameplay
// semantics stay in the in-process harness. (refs: DL-002, DL-005)

use std::io::{self, BufRead, Write};

use crate::{AutomationCommand, AutomationHarness, AutomationSnapshot};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationRequest {
    pub command: AutomationCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationResponse {
    pub snapshot: Option<AutomationSnapshot>,
    pub error: Option<AutomationTransportError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationTransportError {
    pub code: AutomationTransportErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationTransportErrorCode {
    InvalidRequest,
    CommandRejected,
}

impl AutomationResponse {
    fn success(snapshot: AutomationSnapshot) -> Self {
        Self {
            snapshot: Some(snapshot),
            error: None,
        }
    }

    fn invalid_request(message: String) -> Self {
        Self {
            snapshot: None,
            error: Some(AutomationTransportError {
                code: AutomationTransportErrorCode::InvalidRequest,
                message,
            }),
        }
    }

    fn command_rejected(message: String) -> Self {
        Self {
            snapshot: None,
            error: Some(AutomationTransportError {
                code: AutomationTransportErrorCode::CommandRejected,
                message,
            }),
        }
    }
}

pub fn run_stdio_session<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    harness: &mut AutomationHarness,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<AutomationRequest>(&line) {
            Ok(request) => match harness.try_submit(request.command) {
                Ok(snapshot) => AutomationResponse::success(snapshot),
                Err(error) => AutomationResponse::command_rejected(error.to_string()),
            },
            Err(error) => AutomationResponse::invalid_request(error.to_string()),
        };

        serde_json::to_writer(&mut writer, &response).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}
