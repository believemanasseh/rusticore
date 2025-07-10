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
