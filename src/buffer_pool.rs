use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A simple buffer pool implementation.
struct BufferPool {
    /// The maximum size of the buffer pool.
    pub max_size: u8,
    /// The current size of the buffer pool.
    pub current_size: u8,
    /// A thread-safe queue of buffers.
    pub buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl BufferPool {
    /// Creates a new `BufferPool` instance with the specified maximum size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - The maximum number of buffers in the pool.
    ///
    /// # Returns
    ///
    /// A new `BufferPool` instance initialised with empty buffers.
    pub fn new(max_size: u8) -> Self {
        let mut buffers = VecDeque::new();
        for i in 0..max_size {
            let buffer = Vec::new();
            buffers.push_back(buffer);
        }
        BufferPool {
            max_size,
            current_size: max_size,
            buffers: Arc::new(Mutex::new(buffers)),
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
        Some(self.buffers.lock().unwrap().pop_front().unwrap())
    }

    /// Releases a buffer back to the pool.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to be released back to the pool.
    pub fn release(&mut self, buffer: Vec<u8>) {
        todo!("Implement buffer release logic");
    }
}
