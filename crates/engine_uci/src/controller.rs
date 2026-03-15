use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRequest {
    pub position_notation: String,
    pub movetime_millis: u64,
}

impl EngineRequest {
    #[must_use]
    pub fn new(position_notation: impl Into<String>, movetime_millis: u64) -> Self {
        Self {
            position_notation: position_notation.into(),
            movetime_millis,
        }
    }

    pub fn validate(&self) -> Result<(), EngineError> {
        if self.position_notation.trim().is_empty() {
            return Err(EngineError::new("position_notation must not be empty"));
        }
        if self.movetime_millis == 0 {
            return Err(EngineError::new(
                "movetime_millis must be greater than zero",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineResponse {
    pub bestmove_uci: Option<String>,
    pub info: String,
}

impl EngineResponse {
    #[must_use]
    pub fn bestmove(bestmove_uci: impl Into<String>, info: impl Into<String>) -> Self {
        Self {
            bestmove_uci: Some(bestmove_uci.into()),
            info: info.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    message: String,
}

impl EngineError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for EngineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EngineError {}

pub trait EngineController {
    fn name(&self) -> &str;

    fn is_healthy(&self) -> bool;

    fn evaluate(&mut self, request: &EngineRequest) -> Result<EngineResponse, EngineError>;
}
