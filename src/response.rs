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
    pub http_version: String,
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
            http_version: self.http_version.clone(),
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
    fn send(&mut self, body: &str) {
        let response_bytes = self.construct_response_bytes(self, body);
        self.tcp_stream
            .lock()
            .unwrap()
            .write_all(&response_bytes)
            .expect("Failed to write response to TCP stream");
    }

    /// Sends an HTML response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the HTML body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn html(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers
            .push(("Content-Type", "text/html; charset=utf-8"));
        self.send(body);
    }

    /// Sends a JSON response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the JSON body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn json(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "application/json"));
        self.send(body);
    }

    /// Sends a plain text response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the plain text body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn text(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers
            .push(("Content-Type", "text/plain; charset=utf-8"));
        self.send(body);
    }

    /// Sends a CSS response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the CSS body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn css(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers
            .push(("Content-Type", "text/css; charset=utf-8"));
        self.send(body);
    }

    /// Sends a JavaScript response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the JavaScript body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn javascript(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers
            .push(("Content-Type", "application/javascript"));
        self.send(body);
    }

    /// Sends an XML response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the XML body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn xml(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers
            .push(("Content-Type", "application/xml; charset=utf-8"));
        self.send(body);
    }

    /// Sends a PDF response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the PDF body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn pdf(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "application/pdf"));
        self.send(body);
    }

    /// Sends a ZIP response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the ZIP body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn zip(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "application/zip"));
        self.send(body);
    }

    /// Sends a PNG image response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the PNG image body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn audio_mp3(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "audio/mpeg"));
        self.send(body);
    }

    /// Sends a MP4 video response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the MP4 video body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn video_mp4(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "video/mp4"));
        self.send(body);
    }

    /// Sends a PNG image response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the PNG image body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn image_png(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "image/png"));
        self.send(body);
    }

    /// Sends a JPEG image response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the JPEG image body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn image_jpeg(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "image/jpeg"));
        self.send(body);
    }

    /// Sends a GIF image response with the appropriate Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `body` - A string slice representing the GIF image body of the response.
    /// * `status_code` - The HTTP status code for the response.
    pub fn image_gif(&mut self, body: &str, status_code: StatusCode) {
        self.status_code = status_code;
        self.headers.push(("Content-Type", "image/gif"));
        self.send(body);
    }
}
