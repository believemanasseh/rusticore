pub mod logging;
pub mod server;

use log::info;
pub use server::Server;

/// Starts the server using default settings.
///
/// # Returns
///
/// `true` if the server started successfully, `false` otherwise.
pub fn run_server(host: String, port: u16, debug: bool, log_output: Option<String>) -> bool {
    let mut server = Server::new(host, port, debug, log_output);
    server.start();
    if server.is_running() {
        let target = if debug { "app::core" } else { "app::none" };
        info!(target: target, "Server started successfully.");
    }
    server.is_running()
}
