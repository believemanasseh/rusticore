use crate::Server;
use http::StatusCode;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
/// Represents an HTTP response that can be sent back to a client.
pub struct Response<'a> {
    /// The HTTP status code of the response.
    pub status_code: StatusCode,
    /// The HTTP version of the response.
    pub http_version: &'static str,
    /// The headers of the response.
    pub headers: Vec<(&'static str, &'static str)>,
    /// An optional TCP stream to which the response will be sent.
    pub tcp_stream: Arc<Mutex<TcpStream>>,
    /// A thread-safe server instance that is handling the response.
    pub server: Arc<&'a mut Server>,
}

impl<'a> Clone for Response<'a> {
    /// Creates a clone of the `Response` object.
    ///
    /// # Returns
    ///
    /// A new `Response` instance with the same status code, HTTP version, headers, TCP stream and server.
    fn clone(&self) -> Self {
        Response {
            status_code: self.status_code,
            http_version: self.http_version,
            headers: self.headers.clone(),
            tcp_stream: self.tcp_stream.clone(),
            server: Arc::clone(&self.server),
        }
    }
}

impl<'a> Response<'a> {
    /// Constructs the HTTP response byte from the provided `Response` object.
    ///
    /// # Arguments
    ///
    /// * `response` - A reference to the `Response` object containing the HTTP response data.
    /// * `body` - A string slice representing the body of the response.
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the complete HTTP response, including the status line, headers, and body.
    pub fn construct_response_bytes(&self, response: &Response, body: &str) -> Vec<u8> {
        let mut response_bytes = Vec::new();

        // Write request line
        response_bytes.extend_from_slice(response.http_version.as_bytes());
        response_bytes.extend_from_slice(b" ");
        response_bytes.extend_from_slice(response.status_code.as_str().as_bytes());
        response_bytes.extend_from_slice(b" ");
        response_bytes.extend_from_slice(
            response
                .status_code
                .canonical_reason()
                .unwrap_or_default()
                .as_bytes(),
        );
        response_bytes.extend_from_slice(b"\r\n");

        // Write headers
        for (key, value) in response.headers.iter() {
            response_bytes.extend_from_slice(key.as_bytes());
            response_bytes.extend_from_slice(b": ");
            response_bytes.extend_from_slice(value.as_bytes());
            response_bytes.extend_from_slice(b"\r\n");
        }

        // End headers and add body
        response_bytes.extend_from_slice(b"\r\n");
        response_bytes.extend_from_slice(body.as_bytes());

        response_bytes
    }

    /// Constructs a new response string from the `Response` instance.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the body of the response.
    pub fn send(&mut self, body: &str) {
        let response_bytes = self.construct_response_bytes(self, body);
        self.tcp_stream
            .lock()
            .unwrap()
            .write_all(&response_bytes)
            .expect("Failed to write response to TCP stream");
    }
}
