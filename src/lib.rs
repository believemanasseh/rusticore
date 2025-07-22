mod buffer_pool;
mod logging;
mod request;
mod response;
mod routing;
mod server;

pub use buffer_pool::BufferPool;
pub use logging::init_logging;
pub use routing::Route;
pub use server::Server;
pub use server::ServerState;

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
/// A `Result` containing the `Server` instance if successful, or an error message if the server fails to start.
pub fn run_server(
    host: &'static str,
    port: u16,
    debug: bool,
    log_output: Option<&'static str>,
) -> Result<Server, &'static str> {
    let mut server = Server::new(host, port, debug, log_output);
    match server.start() {
        Ok(_) => Ok(server),
        Err(e) => Err(e),
    }
}
