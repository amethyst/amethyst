//! CircularBuffer

use std::collections::VecDeque;

///A CircularBuffer that drops the oldest element inserted when full.
pub struct CircularBuffer<A> {
    queue: VecDeque<A>,
}

impl<A> CircularBuffer<A> {
    ///Creates a new CircularBuffer with fixed size
    pub fn new(size: usize) -> Self {
        CircularBuffer {
            queue: VecDeque::with_capacity(size),
        }
    }

    ///Add a value to the CircularBuffer
    ///Returns the popped value if the buffer is full
    pub fn push(&mut self, elem: A) -> Option<A> {
        let out = if self.queue.len() == self.queue.capacity() {
            //front to back <-> oldest to newest
            self.queue.pop_front()
        } else {
            None
        };

        self.queue.push_back(elem);
        out
    }

    ///Get an immutable reference to the values inside the CircularBuffer
    pub fn queue(&self) -> &VecDeque<A> {
        &self.queue
    }
}
