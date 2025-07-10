use log::{error, info};
use rusticore::run_server;

/// The main entry point of the server.
fn main() {
    let result = run_server(String::from("localhost"), 9000, true, None);
    if result {
        info!("Server started successfully.");
    } else {
        error!("Failed to start the server.");
    }
}
