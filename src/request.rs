use log::warn;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

#[allow(dead_code)]
#[derive(Debug)]
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
}

impl Request {
    /// Creates a new `Request` instance by reading the first line of the HTTP request
    /// from the provided `TcpStream`.
    ///
    /// # Arguments
    /// * `stream` - A `TcpStream` reference representing the incoming connection.
    ///
    /// # Returns
    /// A new `Request` instance with the HTTP method and path parsed from the request line.
    pub fn new(stream: &TcpStream) -> Self {
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
        };
        let request_data = dummy.handle_connection(stream);

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
        }
    }

    /// Handles the incoming connection by reading the HTTP request lines from the `TcpStream`.
    ///
    /// # Arguments
    /// * `stream` - A `TcpStream` reference representing the incoming connection.
    ///
    /// # Returns
    /// A `HashMap<String, String>` containing the parsed HTTP request data.
    fn handle_connection(&self, stream: &TcpStream) -> HashMap<String, String> {
        let buf_reader = BufReader::new(stream);
        let http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut method: Option<&str> = None;
        let mut path: Option<&str> = None;
        let mut http_version: Option<&str> = None;
        let mut host: Option<&str> = None;
        let mut connection: Option<&str> = None;
        let mut cookies: Option<&str> = None;
        let mut cache_control: Option<&str> = None;
        let mut user_agent: Option<&str> = None;
        let mut accept: Option<&str> = None;

        let valid_http_methods = vec![
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
}
