//! Types of physical objects.

use super::Renderable;
use renderer::ir::{CommandBuffer, CommandEncoder};
use renderer::types::Buffer;

/// A particle emitter.
pub struct Emitter;

/// A polygon mesh.
pub struct Mesh {
    pub indices: Option<Buffer>,
    pub vertices: Buffer,
}

impl Mesh {
    pub fn new(vertices: Buffer) -> Mesh {
        Mesh {
            indices: None,
            vertices: vertices,
        }
    }

    pub fn new_indexed(indices: Buffer, vertices: Buffer) -> Mesh {
        Mesh {
            indices: Some(indices),
            vertices: vertices,
        }
    }
}

impl Renderable for Mesh {
    fn to_cmdbuf(&self) -> CommandBuffer {
        let mut encoder = CommandEncoder::new().set_buffer(self.vertices);

        if let Some(indices) = self.indices {
            encoder = encoder.set_buffer(indices)
                             .draw_indexed(0, 0, 0);
        } else {
            encoder = encoder.draw(0, 0);
        }

        encoder.finish()
    }
}

/// A 2D sprite.
pub struct Sprite;
