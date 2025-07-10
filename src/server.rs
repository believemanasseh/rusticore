use crate::logging::init_logging;
use crate::request::Request;
use crate::response::Response;
use http::StatusCode;
use log::info;
use std::io::Write;
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
            let mut stream = stream.unwrap();
            info!(target: target, "New connection from {}", stream.peer_addr().unwrap());
            let request = Request::new(&stream);
            info!(target: target, "Received request: {:?}", request);

            let response = Response {
                status_code: StatusCode::OK,
                body: String::from("Hello, World!"),
                http_version: String::from("HTTP/1.1"),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            };

            let response_str = self.construct_response(&response);
            stream.write_all(response_str.as_bytes()).unwrap()
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

    /// Constructs the HTTP response string from the provided `Response` object.
    ///
    /// # Arguments
    /// * `response` - A reference to a `Response` object containing the HTTP response data.
    ///
    /// # Returns
    /// A `String` representing the complete HTTP response formatted as a string.
    fn construct_response(&self, response: &Response) -> String {
        let mut response_str = String::new();

        // Add the HTTP version and status code to the response string
        response_str.push_str(format!("{} ", &response.http_version).as_str());
        response_str.push_str(format!("{} ", response.status_code.as_u16()).as_str());
        response_str.push_str(
            format!(
                "{}\r\n",
                response.status_code.canonical_reason().unwrap_or_default()
            )
            .as_str(),
        );

        // Add headers to the response string
        for (key, value) in response.headers.iter() {
            response_str.push_str(format!("{}: {}\r\n", key, value).as_str());
        }

        // Add body to the response string
        response_str.push_str(format!("\r\n{}", response.body).as_str());

        response_str
    }
}
