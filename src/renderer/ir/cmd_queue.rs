use super::cmd_buffer::CommandBuffer;

/// Queues and sorts CommandBuffers to minimize redundant state changes.
pub struct CommandQueue {
    queue: Vec<CommandBuffer>,
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue { queue: Vec::new() }
    }

    /// Enqueues a finished CommandBuffer.
    pub fn submit(&mut self, buffer: CommandBuffer) {
        self.queue.push(buffer);
    }

    /// Sorts the queue and returns the result, ready for processing by the
    /// backend.
    pub fn sort_and_flush(&mut self) -> Vec<CommandBuffer> {
        self.queue.sort();
        self.queue.drain(..).collect()
    }
}
