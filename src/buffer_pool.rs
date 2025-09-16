use crate::server::Server;
use log::warn;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
/// A simple buffer pool implementation.
pub struct BufferPool {
    /// The maximum size of the buffer pool.
    max_size: u8,
    /// The current size of the buffer pool.
    current_size: u8,
    /// A thread-safe queue of buffers.
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// A thread-safe reference to the server instance that owns this buffer pool.
    server: Arc<Server>,
}

impl BufferPool {
    /// Creates a new `BufferPool` instance with the specified maximum size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - The maximum number of buffers in the pool.
    /// * `server` - A thread-safe mutable reference to the server instance that owns this buffer pool.
    ///
    /// # Returns
    ///
    /// A new `BufferPool` instance initialised with empty buffers.
    pub fn new(max_size: u8, server: Arc<Server>) -> Self {
        let mut buffers = VecDeque::new();

        for _i in 0..max_size {
            let buffer = Vec::new();
            buffers.push_back(buffer);
        }

        BufferPool {
            max_size,
            current_size: max_size,
            buffers: Arc::new(Mutex::new(buffers)),
            server,
        }
    }

    /// Acquires a buffer from the pool.
    ///
    /// # Returns
    ///
    /// An `Option<Vec<u8>>` containing a buffer if available, or `None` if the pool is empty.
    pub fn acquire(&mut self) -> Option<Vec<u8>> {
        if self.current_size == 0 {
            return None;
        }
        self.current_size -= 1;
        if let Some(mut buffer) = self.buffers.lock().unwrap().pop_front() {
            buffer.clear();
            Some(buffer)
        } else {
            // If the buffer queue is empty, return a new buffer
            Some(Vec::new())
        }
    }

    /// Releases a buffer back to the pool.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to be released back to the pool.
    pub fn release(&mut self, buffer: Vec<u8>) {
        let target = if self.server.debug {
            "app::core"
        } else {
            "app::none"
        };
        if self.current_size < self.max_size {
            self.current_size += 1;
            self.buffers.lock().unwrap().push_back(buffer);
        } else {
            warn!(target: target, "Buffer pool is full, discarding buffer!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::Server;

    #[test]
    /// Tests the functionality of the `BufferPool`.
    /// This test checks if a buffer can be acquired from the pool, released back,
    /// and verifies that the pool's size is maintained correctly.
    fn test_buffer_pool() {
        let server = Server::new("localhost", 8080, false, None, None);
        let arc_server = Arc::new(server);
        let mut pool = BufferPool::new(5, arc_server.clone());

        // Acquire a buffer
        let buffer = pool.acquire();
        assert!(buffer.is_some(), "Buffer should be acquired successfully");
        assert!(
            pool.current_size < pool.max_size,
            "Current size should be less than max size"
        );

        // Release the buffer
        if let Some(buf) = buffer {
            pool.release(buf);
        }

        // Check if the pool size is valid after release
        assert_eq!(
            pool.current_size, pool.max_size,
            "Current size should equal max size after release"
        );
    }
}
