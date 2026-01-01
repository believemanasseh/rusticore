use log::{error, info};
use rusticore::run_server;
use rusticore::ServerState;

/// The main entry point of the server.
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let server = run_server("localhost", 9000, true, None, None).await;

    if let Ok(s) = server.as_ref() {
        if s.check_state(ServerState::Running).await.0 {
            info!("Server started successfully.");
        } else {
            error!("Failed to start the server.");
        }
    }

    if let Err(e) = server {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
