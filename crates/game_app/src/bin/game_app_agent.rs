// Dedicated agent entry point for the feature-gated automation transport.
// The GUI binary remains the player-facing startup path. (ref: DL-005)

use std::io::{self, BufReader};

fn main() -> io::Result<()> {
    let mut harness = game_app::AutomationHarness::new(None).with_semantic_automation();
    let stdin = io::stdin();
    let stdout = io::stdout();

    harness.boot_to_main_menu();
    game_app::automation_transport::run_stdio_session(
        BufReader::new(stdin.lock()),
        stdout.lock(),
        &mut harness,
    )
}
