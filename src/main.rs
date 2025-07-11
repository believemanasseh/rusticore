use log::{error, info};
use rusticore::run_server;
use rusticore::ServerState;

/// The main entry point of the server.
fn main() {
    let server = run_server(String::from("localhost"), 9000, true, None);
    if server.check_state(ServerState::Running).0 {
        info!("Server started successfully.");
    } else {
        error!("Failed to start the server.");
    }
}
