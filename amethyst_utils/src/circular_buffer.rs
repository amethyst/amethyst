//! CircularBuffer

use std::collections::VecDeque;

/// A CircularBuffer that drops the oldest element inserted when full.
/// # Example
///
/// ```
/// # use amethyst::utils::circular_buffer::CircularBuffer;
/// # use std::collections::VecDeque;
/// let mut buf = CircularBuffer::<u32>::new(2);
/// assert_eq!(*buf.queue(), VecDeque::<u32>::from(vec![]));
/// assert!(buf.push(1).is_none());
/// assert_eq!(*buf.queue(), VecDeque::<u32>::from(vec![1]));
/// assert!(buf.push(2).is_none());
/// assert_eq!(*buf.queue(), VecDeque::<u32>::from(vec![1, 2]));
/// assert!(buf.push(3).is_some());
/// assert_eq!(*buf.queue(), VecDeque::<u32>::from(vec![2, 3]));
/// assert_eq!(buf.capacity(), 2);
/// ```
#[derive(Debug)]
pub struct CircularBuffer<A> {
    queue: VecDeque<A>,
    cap: usize,
}

impl<A> CircularBuffer<A> {
    ///Creates a new CircularBuffer with fixed size
    pub fn new(size: usize) -> Self {
        CircularBuffer {
            queue: VecDeque::with_capacity(size),
            cap: size,
        }
    }

    ///Add a value to the CircularBuffer
    ///Returns the popped value if the buffer is full
    pub fn push(&mut self, elem: A) -> Option<A> {
        let out = if self.queue.len() == self.cap {
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

    /// Returns the capacity of the circular buffer.
    pub fn capacity(&self) -> usize {
        self.cap
    }
}
