use crate::request::Request;
use crate::response::Response;
use futures::future::BoxFuture;
use http::StatusCode;
use std::fmt;
use std::sync::Arc;

// pub type Handler = Arc<
//     dyn for<'a> Fn(
//             &'a mut Request,
//             &'a mut Response,
//         ) -> Box<dyn Future<Output = ()> + Send + Unpin + 'a>
//         + Send
//         + Sync
//         + 'static,
// >;

pub type Handler = Arc<
    dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> BoxFuture<'a, ()> + Send + Sync + Unpin,
>;

#[allow(dead_code)]
/// Represents a route in a web application.
pub struct Route {
    /// The HTTP method for the route (e.g., GET, POST).
    pub method: &'static str,
    /// The path for the route (e.g., /home).
    pub path: &'static str,
    /// The handler function for the route.
    pub handler: Handler,
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("handler", &"<function>")
            .finish()
    }
}

// Manual Clone implementation using a wrapper
impl Clone for Route {
    fn clone(&self) -> Self {
        Route {
            method: self.method,
            path: self.path,
            handler: Arc::clone(&self.handler),
        }
    }
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
    pub fn new(method: &'static str, path: &'static str, handler: Handler) -> Self {
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
    pub async fn handle(&self, req: &mut Request, res: &mut Response) {
        (self.handler)(req, res).await
    }
}

#[allow(unused_variables)]
/// A simple handler function for the index route.
///
/// # Arguments
///
/// * `req` - A mutable reference to the incoming HTTP request object.
/// * `res` - A mutable reference to the HTTP response object to which the message will be sent.
pub async fn index(req: &mut Request, res: &mut Response) {
    res.text("Welcome to the index page!", StatusCode::OK).await
}
