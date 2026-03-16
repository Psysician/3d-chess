use engine_uci::{EngineController, EngineRequest, MockEngineController};

#[test]
fn request_validation_rejects_blank_positions_and_zero_movetime() {
    let blank = EngineRequest::new("   ", 150)
        .validate()
        .expect_err("blank positions should be rejected");
    assert_eq!(blank.to_string(), "position_notation must not be empty");

    let zero = EngineRequest::new("startpos", 0)
        .validate()
        .expect_err("zero movetime should be rejected");
    assert_eq!(
        zero.to_string(),
        "movetime_millis must be greater than zero"
    );
}

#[test]
fn mock_controller_returns_scripted_bestmove_for_valid_requests() {
    let mut controller = MockEngineController::new("e2e4");
    let response = controller
        .evaluate(&EngineRequest::new("startpos", 150))
        .expect("mock engine should answer valid requests");

    assert_eq!(response.bestmove_uci.as_deref(), Some("e2e4"));
    assert!(response.info.contains("startpos"));
}

#[test]
fn mock_controller_can_surface_health_and_scripted_failures() {
    let mut unhealthy = MockEngineController::new("e2e4").with_health(false);
    assert!(!unhealthy.is_healthy());
    assert_eq!(
        unhealthy
            .evaluate(&EngineRequest::new("startpos", 150))
            .expect_err("unhealthy mock should fail")
            .to_string(),
        "mock engine is unhealthy"
    );

    let mut failing = MockEngineController::new("e2e4").with_failure("uci unavailable");
    assert_eq!(
        failing
            .evaluate(&EngineRequest::new("startpos", 150))
            .expect_err("configured failure should surface")
            .to_string(),
        "uci unavailable"
    );
}

#[test]
fn default_mock_exposes_name_and_reuses_request_validation() {
    let mut controller = MockEngineController::default();
    assert_eq!(controller.name(), "mock-stockfish");

    let error = controller
        .evaluate(&EngineRequest::new("   ", 150))
        .expect_err("blank requests should still be rejected through evaluate");
    assert_eq!(error.to_string(), "position_notation must not be empty");
}
