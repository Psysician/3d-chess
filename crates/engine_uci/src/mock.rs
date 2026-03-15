use crate::{EngineController, EngineError, EngineRequest, EngineResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockEngineController {
    scripted_move: String,
    scripted_error: Option<String>,
    healthy: bool,
}

impl MockEngineController {
    #[must_use]
    pub fn new(scripted_move: impl Into<String>) -> Self {
        Self {
            scripted_move: scripted_move.into(),
            scripted_error: None,
            healthy: true,
        }
    }

    #[must_use]
    pub fn with_health(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }

    #[must_use]
    pub fn with_failure(mut self, message: impl Into<String>) -> Self {
        self.scripted_error = Some(message.into());
        self
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
        request.validate()?;
        if !self.healthy {
            return Err(EngineError::new("mock engine is unhealthy"));
        }
        if let Some(message) = self.scripted_error.as_ref() {
            return Err(EngineError::new(message.clone()));
        }

        Ok(EngineResponse::bestmove(
            self.scripted_move.clone(),
            format!(
                "mock evaluation for '{}' at {} ms",
                request.position_notation, request.movetime_millis
            ),
        ))
    }
}
