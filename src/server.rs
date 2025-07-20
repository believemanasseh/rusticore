use crate::logging::init_logging;
use crate::request::Request;
use crate::response::Response;
use crate::routing::index;
use crate::Route;
use http::StatusCode;
use log::info;
use std::cmp::PartialEq;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, Clone)]
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
    /// A vector of routes that the server will handle.
    pub routes: Vec<Route>,
}

impl Server {
    /// Creates a new instance of the `Server`.
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname or IP address where the server will run.
    /// * `port` - The port number on which the server will listen.
    /// * `debug` - A boolean indicating whether debug mode is enabled.
    /// * `log_output` - An optional string specifying the log output destination.
    ///
    /// # Returns
    ///
    /// A new instance of `Server` initialised with the provided parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusticore::Server;
    /// let mut server = Server::new(String::from("localhost"), 8080, false, None);
    /// ```
    pub fn new(host: String, port: u16, debug: bool, log_output: Option<String>) -> Self {
        Server {
            host,
            port,
            debug,
            log_output,
            state: ServerState::Starting,
            routes: Vec::from([Route::new(String::from("GET"), String::from("/"), index)]),
        }
    }

    /// Starts the server, binding it to the specified host and port.
    /// It initialises logging, listens for incoming connections, and handles requests.
    pub fn start(&mut self) -> Result<(), &'static str> {
        let server_clone = self.to_owned();

        if let Some(ref log) = self.log_output {
            init_logging(Some(log.to_string()), self.debug);
        } else {
            init_logging(None, self.debug);
        }

        info!("Starting server at {}:{}", self.host, self.port);

        let target = if self.debug { "app::core" } else { "app::none" };

        if self.debug {
            info!(target: target, "Debug mode is enabled.");
        }

        if let Some(ref log) = self.log_output {
            info!(target: target, "Logging output to: {log}");
        }

        // Bind the server to the specified host and port.
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();

        self.state = ServerState::Running;
        info!(target: target, "Server state: {:?}", self.state);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            info!(target: target, "New connection from {}", stream.peer_addr().unwrap());

            // Create a new request instance for the incoming connection.
            if let Ok(ref mut req) = Request::new(&stream, self) {
                // Handle the request based on its path.
                if req.path() == "/" {
                    server_clone.render_index_route(req, stream, target);
                } else {
                    info!(target: target, "Handling route: {}", req.path());
                }
            } else {
                return Err("Failed to parse request");
            }
        }

        Ok(())
    }

    /// Adds a new route to the server's routing vector.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to be added, represented as a `Route` object.
    ///
    /// # Notes
    ///
    /// If the route already exists in the server's routing vector, it will not be added again.
    pub fn add_route(&mut self, route: Route) {
        let target = self.get_target();
        if !self.routes.iter().any(|r| r.path == route.path) {
            info!(target: target, "Added new route: {:#?}", route);
            self.routes.push(route);
        } else {
            info!(target: target, "Route already exists: {:#?}", route);
        }
    }

    /// Adds multiple routes to the server's routing vector.
    ///
    /// # Arguments
    ///
    /// * `routes` - A vector of routes to be added, each represented as a `Route` struct.
    pub fn add_routes(&mut self, routes: Vec<Route>) {
        for route in routes {
            self.add_route(route);
        }
    }

    /// Checks the current state of the server.
    ///
    /// # Arguments
    ///
    /// * `state` - The state to check against the server's current state.
    ///
    /// # Returns
    ///
    /// A tuple containing a boolean indicating whether the server's state matches the provided state,
    pub fn check_state(&self, state: ServerState) -> (bool, &ServerState) {
        let states_match = self.state == state;
        (states_match, &self.state)
    }

    /// Gets the target for logging based on the server's debug mode.
    ///
    /// # Returns
    ///
    /// A string slice representing the target for logging.
    fn get_target(&self) -> &str {
        if self.debug { "app::core" } else { "app::none" }
    }

    /// Renders the index route by reusing the initially created `Route` instance and handling it.
    ///
    /// # Arguments
    ///
    /// * `req` - A mutable reference to the `Request` object representing the incoming request.
    /// * `stream` - A `TcpStream` representing the connection to the client.
    /// * `target` - A string slice representing the target for logging.
    fn render_index_route(&self, req: &mut Request, stream: TcpStream, target: &str) {
        info!(target: target, "Rendering index route: {:#?}", self.routes[0]);
        let res = &mut Response {
            status_code: StatusCode::OK,
            http_version: "HTTP/1.1",
            headers: vec![("Content-Type", "text/plain")],
            tcp_stream: stream.try_clone().ok(),
            server: Arc::from(self.to_owned()),
        };
        self.routes[0].handle(req, res)
    }
}
