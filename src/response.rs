use http::StatusCode;

pub struct Response {
    /// The HTTP status code of the response.
    pub status_code: StatusCode,
    /// The HTTP version of the response.
    pub http_version: String,
    /// The headers of the response.
    pub headers: Vec<(String, String)>,
    /// The body of the response.
    pub body: String,
}

impl Response {
    /// Constructs the HTTP response string from the provided `Response` object.
    ///
    /// # Arguments
    /// * `response` - A reference to a `Response` object containing the HTTP response data.
    ///
    /// # Returns
    /// A `String` representing the complete HTTP response formatted as a string.
    pub fn construct_response_str(&self, response: &Response) -> String {
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
