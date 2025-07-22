use crate::{BufferPool, Server};
use http::method::Method;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
/// Represents a span of text in the HTTP request, defined by its start position and length.
struct Span {
    start: usize,
    length: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Represents an HTTP request parsed from a `TcpStream`.
pub struct Request<'a> {
    /// The HTTP method (e.g., GET, POST) of the request.
    method: Option<Span>,
    /// The route or path requested (e.g., /index).
    path: Option<Span>,
    /// The HTTP version used in the request (e.g., HTTP/1.1).
    http_version: Option<Span>,
    /// The span of the HTTP headers in the request.
    headers: Option<Vec<(Span, Span)>>,
    /// The buffer containing the raw HTTP request data.
    buffer: Vec<u8>,
    /// A thread-safe buffer pool used to manage memory for request buffers.
    buffer_pool: Arc<Mutex<BufferPool<'a>>>,
    /// The cursor position in the buffer, used for parsing.
    cursor: usize,
    /// A thread-safe server instance that is handling the request.
    server: Arc<&'a mut Server>,
}

impl<'a> Drop for Request<'a> {
    /// Releases the buffer back to the buffer pool when the `Request` instance is dropped.
    ///
    /// # Note
    ///
    /// This method ensures that the buffer is returned to the pool for reuse, preventing memory leaks.
    fn drop(&mut self) {
        let mut pool = self.buffer_pool.lock().unwrap();
        pool.release(std::mem::take(&mut self.buffer));
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` instance by reading the HTTP request from the
    /// provided `TcpStream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` or `MockStream` mutable reference representing the incoming connection.
    /// * `server` - A thread-safe mutable reference to the `Server` instance that will handle the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Request` instance if successful, or an error message if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error message if the request cannot be parsed, such as if the connection is closed by the peer,
    /// if there is an error reading from the stream, or if the headers are too large.
    pub fn new<T: Read + Write>(
        stream: &mut T,
        server: Arc<&'a mut Server>,
    ) -> Result<Self, &'static str> {
        let res = Request::parse(stream, server);
        match res {
            Ok(req) => Ok(req),
            Err(e) => Err(e),
        }
    }

    /// Handles the incoming connection by reading the HTTP request lines and headers from the `TcpStream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` or `MockStream` mutable reference representing the incoming connection.
    /// * `server` - A thread-safe mutable reference to the `Server` instance that will handle the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Request` instance if successful, or an error message if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error message if the request cannot be parsed, such as if the connection is closed by the peer,
    /// if there is an error reading from the stream, or if the headers are too large.
    fn parse<T: Read + Write>(
        stream: &mut T,
        server: Arc<&'a mut Server>,
    ) -> Result<Request<'a>, &'static str> {
        let mut request = Request {
            method: None,
            path: None,
            http_version: None,
            headers: Some(Vec::new()),
            buffer: Vec::new(),
            buffer_pool: Arc::new(Mutex::new(BufferPool::new(10, server.clone()))),
            cursor: 0,
            server,
        };

        let mut buf_reader = BufReader::new(stream);
        let mut headers_len = 0;

        if let Some(buffer) = request.buffer_pool.lock().unwrap().acquire() {
            request.buffer = buffer;
        } else {
            return Err("Failed to acquire buffer from pool");
        }

        loop {
            let bytes = match buf_reader.read_until(b'\n', request.buffer.as_mut()) {
                Ok(0) => Err("Connection closed by peer"),
                Ok(n) => Ok(n),
                Err(_) => Err("Error reading from stream"),
            };

            if let Ok(size) = bytes {
                headers_len += size;
            } else {
                return Err("Failed to read from stream");
            }

            // Checks for the end of the headers section
            if request.buffer.ends_with(b"\r\n\r\n") {
                break;
            }

            if headers_len > 4096 {
                return Err("Headers too large");
            }
        }

        // Parse request line (e.g., "GET /path HTTP/1.1")
        if let Some(line_end) = request.buffer[request.cursor..]
            .iter()
            .position(|&b| b == b'\n')
        {
            let line = &request.buffer[request.cursor..request.cursor + line_end];
            let mut parts = line.split(|&b| b == b' ');

            if let Some(method_bytes) = parts.next() {
                request.method = Some(Span {
                    start: request.cursor,
                    length: method_bytes.len(),
                });
            }
            let method_len = request.method.as_ref().map_or(0, |s| s.length);

            if let Some(path_bytes) = parts.next() {
                request.path = Some(Span {
                    start: request.cursor + method_len + 1,
                    length: path_bytes.len(),
                });
            }

            if let Some(version_bytes) = parts.next() {
                request.http_version = Some(Span {
                    start: request.cursor
                        + method_len
                        + 2
                        + request.path.as_ref().map_or(0, |s| s.length),
                    length: version_bytes.len() - 1,
                });
            }

            request.cursor += line_end + 1; // Move cursor to headers start
        } else {
            return Err("Invalid request line");
        }

        // Parse headers
        loop {
            if let Some(line_end) = request.buffer[request.cursor..]
                .iter()
                .position(|&b| b == b'\n')
            {
                let line = &request.buffer[request.cursor..request.cursor + line_end];

                if line.is_empty() || line == b"\r" {
                    request.cursor += line_end + 3; // Move cursor to the request body
                    break; // End of headers
                }

                if let Some(colon_pos) = line.iter().position(|&b| b == b':') {
                    let key = Span {
                        start: request.cursor,
                        length: colon_pos,
                    };
                    let value_start = request.cursor + colon_pos + 2; // Skip ": "
                    let value_end = line.iter().position(|&b| b == b'\r').unwrap_or(line.len());
                    let value = Span {
                        start: value_start,
                        length: value_end - colon_pos - 2,
                    };

                    request.headers.as_mut().unwrap().push((key, value));
                }

                request.cursor += line_end + 1; // Move cursor to the next line
            }
        }

        Ok(request)
    }

    /// Returns the HTTP path of the request.
    ///
    /// # Returns
    ///
    /// A string slice representing the path of the request.
    pub fn path(&self) -> &str {
        if let Some(span) = &self.path {
            std::str::from_utf8(&self.buffer[span.start..span.start + span.length])
                .expect("Invalid UTF-8 in path")
        } else {
            panic!("Path not found in request");
        }
    }

    /// Returns the HTTP method of the request.
    ///
    /// # Returns
    ///
    /// The HTTP method variant (e.g., Method::GET, Method::POST).
    pub fn method(&self) -> Method {
        if let Some(span) = &self.method {
            std::str::from_utf8(&self.buffer[span.start..span.start + span.length])
                .expect("Invalid UTF-8 in method")
                .parse::<Method>()
                .expect("Failed to parse HTTP method")
        } else {
            panic!("Method not found in request");
        }
    }

    /// Returns the HTTP version of the request.
    ///
    /// # Returns
    ///
    /// A string slice representing the HTTP version (e.g., HTTP/1.1, HTTP/2).
    pub fn http_version(&self) -> &str {
        if let Some(span) = &self.http_version {
            std::str::from_utf8(&self.buffer[span.start..span.start + span.length])
                .expect("Invalid UTF-8 in HTTP version")
        } else {
            panic!("HTTP version not found in request");
        }
    }

    /// Returns the raw bytes for any content type in the HTTP request.
    ///
    /// # Returns
    ///
    /// A slice of bytes representing the body of the request.
    pub fn body(&self) -> &[u8] {
        &self.buffer[self.cursor..]
    }

    /// Returns the value of a specific header from the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the header key to look for (case-insensitive).
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the value of the header if found, or `None` if the header does not exist.
    pub fn get_header(&self, key: &str) -> Option<&str> {
        if let Some(headers) = &self.headers {
            for (k, v) in headers {
                if let Ok(header_key) =
                    std::str::from_utf8(&self.buffer[k.start..k.start + k.length])
                {
                    if header_key.eq_ignore_ascii_case(key) {
                        return Some(
                            std::str::from_utf8(&self.buffer[v.start..v.start + v.length]).unwrap(),
                        );
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock_io::sync::{MockListener, MockStream};
    use std::thread;

    #[test]
    /// Tests the parsing of an HTTP request from a `MockStream` (`std::net::TcpStream` equivalent).
    /// It simulates a client sending a request and checks if the `Request` struct is correctly populated
    /// with the method, path, HTTP version, and headers.
    fn test_request_parsing() {
        let mut server = Server::new("localhost", 8080, false, None);
        let arc_server = Arc::new(&mut server);
        let request_data = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let (listener, handle) = MockListener::new();

        // Spawn a thread to simulate a client sending a request
        thread::spawn(move || {
            let mut stream = MockStream::connect(&handle).unwrap();
            stream.write(request_data).unwrap();
        });

        while let Ok(mut stream) = listener.accept() {
            match Request::parse(&mut stream, arc_server.clone()) {
                Ok(request) => {
                    assert_eq!(request.method(), Method::GET);
                    assert_eq!(request.path(), "/");
                    assert_eq!(request.http_version(), "HTTP/1.1");
                    assert_eq!(request.get_header("Host"), Some("localhost"));
                }
                Err(e) => assert!(false, "Failed to parse request: {e}"),
            }
        }
    }
}
