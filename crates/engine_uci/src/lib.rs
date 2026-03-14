mod controller;
mod mock;

pub use controller::{EngineController, EngineError, EngineRequest, EngineResponse};
pub use mock::MockEngineController;

#[cfg(test)]
mod tests {
    use crate::{EngineController, EngineRequest, MockEngineController};

    #[test]
    fn mock_engine_returns_the_scripted_move() {
        let mut controller = MockEngineController::new("e2e4");
        let response = controller
            .evaluate(&EngineRequest::new("startpos", 150))
            .expect("the mock controller should always be able to answer");

        assert_eq!(response.bestmove_uci.as_deref(), Some("e2e4"));
    }
}
