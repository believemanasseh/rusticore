use crate::request::Request;
use crate::response::Response;
use http::StatusCode;

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Represents a route in a web application.
pub struct Route {
    /// The HTTP method for the route (e.g., GET, POST).
    pub method: &'static str,
    /// The path for the route (e.g., /home).
    pub path: &'static str,
    /// The handler function for the route.
    pub handler: fn(&mut Request, &mut Response),
}

impl Route {
    /// Constructs a new `Route` instance.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method for the route (e.g., GET, POST).
    /// * `path` - The path for the route (e.g., /home).
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A new `Route` instance.
    pub fn new(
        method: &'static str,
        path: &'static str,
        handler: fn(&mut Request, &mut Response),
    ) -> Self {
        Route {
            method,
            path,
            handler,
        }
    }

    /// Handles the route by calling the associated handler function.
    ///
    /// # Arguments
    ///
    /// * `req` - A mutable reference to the incoming HTTP request object.
    /// * `res` - A mutable reference to the HTTP response object to which the handler will send the response.
    pub fn handle(&self, req: &mut Request, res: &mut Response) {
        (self.handler)(req, res)
    }
}

#[allow(unused_variables)]
/// A simple handler function for the index route.
///
/// # Arguments
///
/// * `req` - A mutable reference to the incoming HTTP request object.
/// * `res` - A mutable reference to the HTTP response object to which the message will be sent.
pub fn index(req: &mut Request, res: &mut Response) {
    res.text("Welcome to the index page!", StatusCode::OK)
}
