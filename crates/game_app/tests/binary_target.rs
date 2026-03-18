use std::path::Path;

#[test]
fn game_app_binary_is_built_for_integration_tests() {
    let binary_path = env!("CARGO_BIN_EXE_game_app");

    assert!(Path::new(binary_path).exists());
}

#[cfg(feature = "automation-transport")]
#[test]
fn game_app_agent_binary_is_built_when_transport_feature_is_enabled() {
    let binary_path = env!("CARGO_BIN_EXE_game_app_agent");

    assert!(Path::new(binary_path).exists());
}
