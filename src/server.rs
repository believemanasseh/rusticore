use crate::logging::init_logging;
use crate::request::Request;
use crate::response::Response;
use crate::routing::{index, Handler};
use crate::Route;
use http::StatusCode;
use log::info;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
/// Represents the possible states of a server.
pub enum ServerState {
    /// The server is in the process of starting up.
    Starting,
    /// The server is currently running and accepting requests.
    Running,
    /// The server is in th e process of stopping.
    Stopping,
    /// The server has been stopped and is not running.
    Stopped,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Represents a server configuration with various parameters.
pub struct Server {
    /// The hostname or IP address where the server will run.
    pub host: &'static str,
    /// The port number on which the server will listen.
    pub port: u16,
    /// A boolean indicating whether debug mode is enabled.
    pub debug: bool,
    /// An optional string specifying the log output destination.
    pub log_output: Option<&'static str>,
    /// The current state of the server.
    pub state: Arc<Mutex<ServerState>>,
    /// A vector of routes that the server will handle.
    pub routes: Arc<RwLock<Vec<Route>>>,
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
    /// * `default_index_handler` - An optional function to handle the index route. If not provided, a default handler will be used.
    ///
    /// # Returns
    ///
    /// A new instance of `Server` initialised with the provided parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusticore::Server;
    /// let mut server = Server::new("localhost", 8080, false, None, None);
    /// ```
    pub fn new(
        host: &'static str,
        port: u16,
        debug: bool,
        log_output: Option<&'static str>,
        default_index_handler: Option<Handler>,
    ) -> Self {
        // Use the provided index handler or default to the built-in index handler.
        let index_handler: Handler;
        if let Some(handler) = default_index_handler {
            index_handler = handler
        } else {
            index_handler = Arc::new(|req, res| Box::pin(index(req, res)));
        }

        Server {
            host,
            port,
            debug,
            log_output,
            state: Arc::new(Mutex::new(ServerState::Starting)),
            routes: Arc::new(RwLock::new(Vec::from([Route::new(
                "GET",
                "/",
                index_handler,
            )]))),
        }
    }

    /// Starts the server, binding it to the specified host and port.
    /// It initialises logging, listens for incoming connections, and handles requests.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the server start operation.
    pub async fn start(&mut self) -> Result<(), &'static str> {
        if let Some(log) = self.log_output {
            init_logging(Some(log), self.debug);
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
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port))
            .await
            .unwrap();

        let arc_server = Arc::new(self.clone());

        let mut state = arc_server.state.lock().await;
        *state = ServerState::Running;
        info!(target: target, "Server state: {:?}", *state);

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            info!(target: target, "New connection from {}", stream.peer_addr().unwrap());
            let arc_server = arc_server.clone();

            tokio::spawn(async move {
                // Create a new request instance for the incoming connection.
                let server = arc_server.clone();
                let mut req = match Request::new(&mut stream, server).await {
                    Ok(r) => r,
                    Err(_) => return,
                };

                // Handle the request based on its path.
                let routes = arc_server.routes.read().await;
                for route in routes.iter() {
                    let (matched, query_params, path_params) =
                        Server::match_route(route.path, req.path());

                    if matched {
                        req.query_params = query_params;
                        req.path_params = path_params;

                        info!(target: target, "Handling route: {}", req.path());
                        let res = &mut Response {
                            status_code: StatusCode::OK,
                            http_version: Arc::new(req.http_version().to_string()),
                            headers: vec![],
                            tcp_stream: Arc::new(Mutex::new(stream)),
                            server: arc_server.clone(),
                        };
                        route.handle(&mut req, res).await;
                        break;
                    }
                }
            });
        }
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
    pub async fn add_route(&mut self, route: Route) {
        let mut routes = self.routes.write().await;
        let target = self.get_target();
        if !routes.iter().any(|r| r.path == route.path) {
            info!(target: target, "Added new route: {}", route.path);
            routes.push(route);
        } else {
            info!(target: target, "Route already exists: {}", route.path);
        }
    }

    /// Adds multiple routes to the server's routing vector.
    ///
    /// # Arguments
    ///
    /// * `routes` - A vector of routes to be added, each represented as a `Route` struct.
    pub async fn add_routes(&mut self, routes: Vec<Route>) {
        for route in routes {
            self.add_route(route).await;
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
    pub async fn check_state(&self, state: ServerState) -> (bool, ServerState) {
        let guard = self.state.lock().await;
        let current = guard.clone();
        (current == state, current)
    }

    /// Gets the target for logging based on the server's debug mode.
    ///
    /// # Returns
    ///
    /// A string slice representing the target for logging.
    fn get_target(&self) -> &str {
        if self.debug { "app::core" } else { "app::none" }
    }

    /// Matches a given route pattern against a path and extracts query and path parameters.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The route pattern, e.g., "/users/{id}".
    /// * `path` - The actual path to match, e.g., "/users/42".
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A boolean indicating whether the pattern matches the path.
    /// * A `HashMap` of query parameters extracted from the path.
    /// * A `HashMap` of path parameters extracted from the path.
    fn match_route(
        pattern: &str,
        path: &str,
    ) -> (bool, HashMap<String, String>, HashMap<String, String>) {
        let mut query_params = HashMap::new();
        let mut path_params = HashMap::new();
        let parts: Vec<&str> = path.split('?').collect();

        if parts.len() == 2 {
            let query_string = parts[1];
            for param in query_string.split('&') {
                let kv: Vec<&str> = param.split('=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        let route_parts: Vec<&str> = pattern.trim_end_matches('/').split('/').collect();
        let path_parts: Vec<&str> = parts[0].trim_end_matches('/').split('/').collect();

        if route_parts.len() != path_parts.len() {
            return (false, query_params, path_params);
        }

        for (pat, val) in route_parts.iter().zip(path_parts.iter()) {
            if pat.starts_with('{') && pat.ends_with('}') {
                let key = &pat[1..pat.len() - 1];
                path_params.insert(key.to_string(), val.to_string());
            } else if pat != val {
                return (false, query_params, path_params);
            }
        }

        (true, query_params, path_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    /// Tests the creation of a new server instance with default parameters.
    /// It checks that the server's host, port, debug mode, log output, state, and routes are initialised correctly.
    /// It also verifies that the initial index route is set.
    async fn new() {
        let server = Server::new("localhost", 8080, false, None, None);
        assert_eq!(server.host, "localhost");
        assert_eq!(server.port, 8080);
        assert!(!server.debug);
        assert!(server.log_output.is_none());
        let (matches, current) = server.check_state(ServerState::Starting).await;
        assert!(matches);
        assert_eq!(current, ServerState::Starting);
        let routes = server.routes.read().await;
        assert_eq!(routes.len(), 1);
    }

    #[tokio::test]
    /// Tests the addition of single routes to the server.
    /// It checks that a new route has been added to the server's routing vector,
    async fn add_route() {
        let server = &mut Server::new("localhost", 8080, false, None, None);
        server
            .add_route(Route::new(
                "GET",
                "/test",
                Arc::new(|req, res| Box::pin(index(req, res))),
            ))
            .await;
        let routes = server.routes.read().await;
        assert_eq!(routes.len(), 2);
    }

    #[tokio::test]
    /// Tests the addition of multiple routes to the server.
    /// It checks that the server's routing vector has been updated with the new routes,
    async fn add_routes() {
        let server = &mut Server::new("localhost", 8080, false, None, None);
        let routes = vec![
            Route::new(
                "GET",
                "/test1",
                Arc::new(|req, res| Box::pin(index(req, res))),
            ),
            Route::new(
                "PUT",
                "/test3",
                Arc::new(|req, res| Box::pin(index(req, res))),
            ),
        ];
        server.add_routes(routes).await;
        let routes = server.routes.read().await;
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    /// Tests the route matching functionality of the server.
    /// It checks that a given route pattern matches a path and correctly extracts query and path parameters.
    async fn match_route() {
        let (matched, query_params, path_params) =
            Server::match_route("/users/{id}", "/users/42?key=value");
        assert!(matched);
        assert!(path_params.contains_key("id"));
        assert_eq!(path_params.get("id").unwrap(), "42");
        assert!(query_params.contains_key("key"));
        assert_eq!(query_params.get("key").unwrap(), "value");
    }
}
