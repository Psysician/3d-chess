use crate::{EngineController, EngineError, EngineRequest, EngineResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockEngineController {
    scripted_move: String,
    healthy: bool,
}

impl MockEngineController {
    #[must_use]
    pub fn new(scripted_move: impl Into<String>) -> Self {
        Self {
            scripted_move: scripted_move.into(),
            healthy: true,
        }
    }
}

impl Default for MockEngineController {
    fn default() -> Self {
        Self::new("e2e4")
    }
}

impl EngineController for MockEngineController {
    fn name(&self) -> &str {
        "mock-stockfish"
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }

    fn evaluate(&mut self, request: &EngineRequest) -> Result<EngineResponse, EngineError> {
        if request.position_notation.trim().is_empty() {
            return Err(EngineError::new("position_notation must not be empty"));
        }

        Ok(EngineResponse {
            bestmove_uci: Some(self.scripted_move.clone()),
            info: format!(
                "mock evaluation for '{}' at {} ms",
                request.position_notation, request.movetime_millis
            ),
        })
    }
}
