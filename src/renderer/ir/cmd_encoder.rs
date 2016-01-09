use super::cmd_buffer::{Command, CommandBuffer, SortKey};
use super::state_dynamic::DynamicState;
use super::state_static::Pipeline;
use renderer::types::{Buffer, ClearMask};

/// Builds new CommandBuffers.
pub struct CommandEncoder {
    buffer: Vec<Command>,
}

impl CommandEncoder {
    pub fn new() -> CommandEncoder {
        CommandEncoder { buffer: Vec::new(), }
    }

    /// Signals that the encoder has finished recording.
    pub fn finish(&self) -> CommandBuffer {
        // Placeholder
        let sort_key: SortKey = 1;
        CommandBuffer::new(self.buffer.clone(), sort_key)
    }

    /// Clears the specified buffer to a value.
    pub fn clear(mut self, mask: ClearMask, value: [f32; 4]) -> Self {
        self.buffer.push(Command::Clear(mask, value));
        self
    }

    /// Draws non-indexed, non-instanced primitives.
    pub fn draw(mut self, first_vertex: u32, count: u32) -> Self {
        self.buffer.push(Command::Draw(first_vertex, count));
        self
    }

    /// Draws indexed, non-instanced primitives.
    pub fn draw_indexed(mut self, first: u32, count: u32, vertex_offset: usize) -> Self {
        self.buffer.push(Command::DrawIndexed(first, count, vertex_offset));
        self
    }

    /// Binds a vertex/index/constant buffer to pull data from.
    pub fn set_buffer(mut self, handle: Buffer) -> Self {
        self.buffer.push(Command::SetBuffer(handle));
        self
    }

    /// Binds dynamic state (blend, depth-stencil, rasterizer, or viewport).
    pub fn set_dynamic_state(mut self, handle: DynamicState) -> Self {
        self.buffer.push(Command::SetDynamicState(handle));
        self
    }

    /// Binds a static pipeline state object to use when drawing.
    pub fn set_pipeline(mut self, handle: Pipeline) -> Self {
        self.buffer.push(Command::SetPipeline(handle));
        self
    }
}
