use super::logging::init_logging;
use log::info;
use std::net::TcpListener;

#[allow(dead_code)]
#[derive(Debug)]
/// Represents the possible states of a server.
pub enum ServerState {
    /// The server is in the process of starting up.
    Starting,
    /// The server is currently running and accepting requests.
    Running,
    /// The server is in the process of stopping.
    Stopping,
    /// The server has been stopped and is not running.
    Stopped,
}

#[allow(dead_code)]
/// Represents a server configuration with various parameters.
pub struct Server {
    /// The hostname or IP address where the server will run.
    pub host: String,
    /// The port number on which the server will listen.
    pub port: u16,
    /// A boolean indicating whether debug mode is enabled.
    pub debug: bool,
    /// An optional string specifying the log output destination.
    pub log_output: Option<String>,
    /// The current state of the server.
    pub state: ServerState,
}

impl Server {
    /// Creates a new instance of the `Server`.
    ///
    /// # Arguments
    /// * `host` - The hostname or IP address where the server will run.
    /// * `port` - The port number on which the server will listen.
    /// * `debug` - A boolean indicating whether debug mode is enabled.
    /// * `log_output` - An optional string specifying the log output destination.
    ///
    /// # Returns
    /// A new instance of `Server` initialised with the provided parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rusticore::server::Server;
    /// let mut server = Server::new(String::from("localhost"), 8080, false, None);
    /// ```
    pub fn new(host: String, port: u16, debug: bool, log_output: Option<String>) -> Self {
        Server {
            host,
            port,
            debug,
            log_output,
            state: ServerState::Starting,
        }
    }

    /// Starts the server and logs the status.
    pub fn start(&mut self) {
        if let Some(ref log) = self.log_output {
            init_logging(Some(log.to_string()), self.debug);
        } else {
            init_logging(None, self.debug);
        }

        let target = if self.debug { "app::core" } else { "app::none" };

        info!("Starting server at {}:{}", self.host, self.port);

        if self.debug {
            info!(target: target, "Debug mode is enabled.");
        }

        if let Some(ref log) = self.log_output {
            info!(target: target, "Logging output to: {log}");
        }

        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();

        self.state = ServerState::Running;
        info!(target: target, "Server state: {:?}", self.state);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            info!(target: target, "New connection from {}", stream.peer_addr().unwrap());
        }
    }

    /// Checks if the server is currently running.
    ///
    /// # Returns
    /// `true` if the server is running, `false` otherwise.
    pub fn is_running(&self) -> bool {
        let target = self.get_target();
        info!(target: target, "Server is running at {}:{}", self.host, self.port);
        matches!(self.state, ServerState::Running)
    }

    /// Gets the target for logging based on the server's debug mode.
    ///
    /// # Returns
    /// A string slice representing the target for logging.
    fn get_target(&self) -> &str {
        if self.debug { "app::core" } else { "app::none" }
    }
}
