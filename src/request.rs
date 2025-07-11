use crate::Server;
use log::{error, warn};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Represents an HTTP request parsed from a `TcpStream`.
pub struct Request {
    /// The HTTP method (e.g., GET, POST) of the request.
    pub method: Option<String>,
    /// The route or path requested (e.g., /index).
    pub path: Option<String>,
    /// The HTTP version used in the request (e.g., HTTP/1.1).
    pub http_version: Option<String>,
    /// The host header from the request, indicating the server's hostname.
    pub host: Option<String>,
    /// The connection header, indicating whether the connection should be kept alive.
    pub connection: Option<String>,
    /// The cookies sent with the request, if any.
    pub cookies: Option<String>,
    /// The cache control header, indicating caching policies.
    pub cache_control: Option<String>,
    /// The accept header, indicating the media types that are acceptable for the response.
    pub accept: Option<String>,
    /// The user agent header, indicating the client software making the request.
    pub user_agent: Option<String>,
    /// The server instance that is handling the request.
    pub server: Arc<Server>,
}

impl Request {
    /// Creates a new `Request` instance by reading the first line of the HTTP request
    /// from the provided `TcpStream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` mutable reference representing the incoming connection.
    ///
    /// # Returns
    ///
    /// A tuple containing a `Request` instance and a `HashMap<String, String>` with the parsed request data.
    pub fn new(stream: &TcpStream, server: &mut Server) -> (Self, HashMap<String, String>) {
        let dummy = Self {
            method: None,
            path: None,
            http_version: None,
            host: None,
            connection: None,
            cookies: None,
            cache_control: None,
            accept: None,
            user_agent: None,
            server: Arc::from(server.to_owned()),
        };
        let request_data = dummy.handle_connection(stream);
        let request_obj = dummy.convert_hashmap_to_request(&request_data);
        (request_obj, request_data)
    }

    /// Handles the incoming connection by reading the HTTP request lines from the `TcpStream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` reference representing the incoming connection.
    ///
    /// # Returns
    ///
    /// A `HashMap<String, String>` containing the parsed HTTP request data.
    pub fn handle_connection(&self, stream: &TcpStream) -> HashMap<String, String> {
        let target = if self.server.debug {
            "app::core"
        } else {
            "app::none"
        };
        let mut buf_reader = BufReader::new(stream);
        let mut http_request = Vec::new();

        for line in buf_reader.by_ref().lines() {
            match line {
                Ok(l) if !l.is_empty() => http_request.push(l),
                Ok(_) => break,
                Err(e) => {
                    error!(target: target, "Error reading from stream: {}", e);
                    break;
                }
            }
        }

        let mut method: Option<&str> = None;
        let mut path: Option<&str> = None;
        let mut http_version: Option<&str> = None;
        let mut host: Option<&str> = None;
        let mut connection: Option<&str> = None;
        let mut cookies: Option<&str> = None;
        let mut cache_control: Option<&str> = None;
        let mut user_agent: Option<&str> = None;
        let mut accept: Option<&str> = None;

        let valid_http_methods: [&str; 8] = [
            "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE",
        ];

        for line in http_request.iter() {
            if line.contains("HTTP/") {
                let mut parts = line.split_whitespace();
                method = parts.next();
                path = parts.next();
                http_version = parts.next()
            } else if line.contains("Host:") {
                host = line.split_whitespace().nth(1)
            } else if line.contains("Connection:") {
                connection = line.split_whitespace().nth(1)
            } else if line.contains("Cache-Control:") {
                cache_control = line.split_whitespace().nth(1)
            } else if line.contains("Accept:") {
                accept = line.split_whitespace().nth(1)
            } else if line.contains("User-Agent:") {
                user_agent = Option::from(line.split_once(" ").unwrap().1)
            } else if line.contains("Cookie:") {
                cookies = Option::from(line.split_once(" ").unwrap().1)
            }
        }

        if !method.is_none() && !valid_http_methods.contains(&method.unwrap()) {
            panic!("Invalid HTTP method parsed: {}", method.unwrap());
        } else if method.is_none() {
            panic!("No HTTP method found in the request.");
        } else if path.is_none() {
            panic!("No HTTP path found in the request.");
        } else if http_version.is_none() {
            panic!("No HTTP version found in the request.");
        } else if host.is_none() {
            warn!("No Host header found in the request.");
        } else if connection.is_none() {
            warn!("No Connection header found in the request.");
        } else if cookies.is_none() {
            warn!("No Cookies header found in the request.");
        } else if cache_control.is_none() {
            warn!("No Cache-Control header found in the request.");
        }

        HashMap::from([
            ("method".to_string(), method.unwrap().to_string()),
            ("path".to_string(), path.unwrap().to_string()),
            (
                "http_version".to_string(),
                http_version.unwrap().to_string(),
            ),
            ("host".to_string(), host.unwrap().to_string()),
            ("connection".to_string(), connection.unwrap().to_string()),
            ("cookies".to_string(), cookies.unwrap().to_string()),
            (
                "cache_control".to_string(),
                cache_control.unwrap_or_default().to_string(),
            ),
            (
                "user_agent".to_string(),
                user_agent.unwrap_or_default().to_string(),
            ),
            ("accept".to_string(), accept.unwrap().to_string()),
        ])
    }

    /// Converts a `HashMap<String, String>` containing request data into a `Request` instance.
    ///
    /// # Arguments
    ///
    /// * `request_data` - A `HashMap<String, String>` reference containing the request data.
    ///
    /// # Returns
    ///
    /// A new `Request` instance populated with the data from the `HashMap`.
    pub fn convert_hashmap_to_request(&self, request_data: &HashMap<String, String>) -> Self {
        Request {
            method: Option::from(request_data["method"].to_string()),
            path: Option::from(request_data["path"].to_string()),
            http_version: Option::from(request_data["http_version"].to_string()),
            host: Option::from(request_data["host"].to_string()),
            connection: Option::from(request_data["connection"].to_string()),
            cookies: Option::from(request_data["cookies"].to_string()),
            cache_control: Option::from(request_data["cache_control"].to_string()),
            accept: Option::from(request_data["accept"].to_string()),
            user_agent: Option::from(request_data["user_agent"].to_string()),
            server: self.server.to_owned(),
        }
    }
}
