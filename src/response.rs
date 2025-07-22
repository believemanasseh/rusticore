use crate::Server;
use http::StatusCode;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

#[derive(Debug)]
/// Represents an HTTP response that can be sent back to a client.
pub struct Response {
    /// The HTTP status code of the response.
    pub status_code: StatusCode,
    /// The HTTP version of the response.
    pub http_version: &'static str,
    /// The headers of the response.
    pub headers: Vec<(&'static str, &'static str)>,
    /// An optional TCP stream to which the response will be sent.
    pub tcp_stream: Option<TcpStream>,
    /// A thread-safe server instance that is handling the response.
    pub server: Arc<Server>,
}

impl Clone for Response {
    /// Creates a clone of the `Response` object.
    ///
    /// # Returns
    ///
    /// A new `Response` instance with the same status code, HTTP version, headers, body, and TCP stream.
    fn clone(&self) -> Self {
        Response {
            status_code: self.status_code,
            http_version: self.http_version,
            headers: self.headers.clone(),
            tcp_stream: self.tcp_stream.as_ref().and_then(|s| s.try_clone().ok()),
            server: Arc::clone(&self.server),
        }
    }
}

impl Response {
    /// Constructs the HTTP response string from the provided `Response` object.
    ///
    /// # Arguments
    ///
    /// * `response` - A reference to a `Response` object containing the HTTP response data.
    ///
    /// # Returns
    ///
    /// A `String` representing the complete HTTP response formatted as a string.
    pub fn construct_response_str(&self, response: &Response, body: &str) -> String {
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
        response_str.push_str(format!("\r\n{}", body).as_str());

        response_str
    }

    /// Constructs a new response string from a `Response` instance.
    ///
    /// # Arguments
    ///
    /// * `body` - The body of the response as a `String`.
    pub fn send(&mut self, body: &str) {
        let response_str = self.construct_response_str(self, body);
        if let Some(ref mut tcp_stream) = self.tcp_stream {
            tcp_stream.write_all(response_str.as_bytes()).unwrap()
        } else {
            panic!("TCP stream is not set. Cannot send response.");
        }
    }
}
