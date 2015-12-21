//! Platform-agnostic intermediate representation used by the frontend and
//! backend to communicate.

use renderer::backend::{DynamicState, Pipeline};
use renderer::types::{Buffer, ClearMask};

/// Serialized graphics command.
#[derive(Clone)]
pub enum Command {
    Clear(ClearMask, [f32; 4]),
    Draw(u32, u32),
    DrawIndexed(u32, u32, usize),
    SetBuffer(Buffer),
    SetDynamicState(DynamicState),
    SetPipeline(Pipeline),
}

/// A trait which adds methods to more easily populate a CommandBuffer.
pub trait AddCommands {
    /// Clears the specified buffer to a value.
    fn clear(&mut self, mask: ClearMask, value: [f32; 4]) -> &mut Self;
    /// Draws non-indexed, non-instanced primitives.
    fn draw(&mut self, first: u32, count: u32) -> &mut Self;
    /// Draws indexed, non-instanced primitives.
    fn draw_indexed(&mut self, first: u32, count: u32, vertex_offset: usize)
                    -> &mut Self;
    /// Binds a vertex/index/constant buffer to pull data from.
    fn set_buffer(&mut self, handle: Buffer) -> &mut Self;
    /// Binds dynamic state (blend, depth-stencil, rasterizer, or viewport).
    fn set_dynamic_state(&mut self, handle: DynamicState) -> &mut Self;
    /// Binds a static pipeline state object to use when drawing.
    fn set_pipeline(&mut self, handle: Pipeline) -> &mut Self;
}

/// A collection of Commands.
pub type CommandBuffer = Vec<Command>;

/// 64-bit key used for sorting CommandBuffers. TODO: Need design for fields.
pub type SortKey = u64;

/// Builds a CommandBuffer and generates an associated SortKey.
#[derive(Clone)]
pub struct CommandEncoder {
    buffer: CommandBuffer,
    pub key: SortKey,
}

impl CommandEncoder {
    pub fn new() -> CommandEncoder {
        CommandEncoder {
            buffer: CommandBuffer::new(),
            key: 0,
        }
    }

    /// Signals that the encoder has finished recording.
    pub fn finish(&mut self) -> CommandEncoder {
        // Placeholder
        self.key = 1;
        self.clone()
    }
}

impl AddCommands for CommandEncoder {
    fn clear(&mut self, mask: ClearMask, value: [f32; 4]) -> &mut Self {
        self.buffer.push(Command::Clear(mask, value));
        self
    }

    fn draw(&mut self, first_vertex: u32, count: u32) -> &mut Self {
        self.buffer.push(Command::Draw(first_vertex, count));
        self
    }

    fn draw_indexed(&mut self, first: u32, count: u32, vertex_offset: usize)
                    -> &mut Self {
        self.buffer.push(Command::DrawIndexed(first, count, vertex_offset));
        self
    }

    fn set_buffer(&mut self, handle: Buffer) -> &mut Self {
        self.buffer.push(Command::SetBuffer(handle));
        self
    }

    fn set_dynamic_state(&mut self, handle: DynamicState) -> &mut Self {
        self.buffer.push(Command::SetDynamicState(handle));
        self
    }

    fn set_pipeline(&mut self, handle: Pipeline) -> &mut Self {
        self.buffer.push(Command::SetPipeline(handle));
        self
    }
}

/// Queues and sorts CommandBuffers to minimize redundant state changes.
pub struct CommandQueue {
    queue: Vec<CommandEncoder>,
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue { queue: Vec::new() }
    }

    /// Enqueues a finished CommandEncoder.
    pub fn submit(&mut self, encoder: CommandEncoder) {
        self.queue.push(encoder);
    }

    /// Sorts the queue and returns the result, ready for processing by the
    /// backend.
    pub fn sort_and_flush(&mut self) -> Vec<CommandBuffer> {
        self.queue.sort_by(|prev, next| {
            prev.key.cmp(&next.key)
        });

        self.queue.drain(..)
                  .map(|encoder| { encoder.buffer })
                  .collect()
    }
}

