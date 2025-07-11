pub mod logging;
mod request;
mod response;
pub mod routing;
pub mod server;

use crate::server::ServerState;
use log::info;
pub use routing::Route;
pub use server::Server;

/// Starts the server using default settings.
///
/// # Arguments
///
/// * `host` - The host address to bind the server to.
/// * `port` - The port number on which the server will listen.
/// * `debug` - A boolean indicating whether debug mode is enabled.
/// * `log_output` - An optional string specifying the log output destination.
///
/// # Returns
///
/// `true` if the server started successfully, `false` otherwise.
pub fn run_server(host: String, port: u16, debug: bool, log_output: Option<String>) -> bool {
    let mut server = Server::new(host, port, debug, log_output);
    server.start();
    let is_running = server.check_state(ServerState::Running).0;
    if is_running {
        let target = if debug { "app::core" } else { "app::none" };
        info!(target: target, "Server started successfully.");
    }
    is_running
}
