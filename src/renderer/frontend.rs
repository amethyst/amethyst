//! Builds IR command buffers from Objects and feeds them into the backend.

use renderer::backend::Backend;
use renderer::ir::{AddCommands, CommandEncoder, CommandQueue};
use renderer::types::Buffer;

/// A light source.
#[derive(Clone)]
pub enum Light {
    Area,
    Directional {
        color: [f32; 4],
        direction: [f32; 3],
        intensity: f32,
    },
    Point {
        color: [f32; 4],
        intensity: f32,
        location: [f32; 3],
    },
    Spot {
        angle: f32,
        color: [f32; 4],
        direction: [f32; 3],
        intensity: f32,
        location: [f32; 3],
    },
}

/// A physical renderable object.
#[derive(Clone)]
pub enum Object {
    Emitter,
    IndexedMesh {
        indices: Buffer,
        vertices: Buffer,
    },
    Mesh {
        vertices: Buffer,
    },
    Sprite,
}

/// A collection of lights and objects to be rendered by the frontend.
pub struct Frame {
    pub lights: Vec<Light>,
    pub objects: Vec<Object>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }
}

/// Simple renderer frontend.
pub struct Frontend {
    backend: Box<Backend>,
    queue: CommandQueue,
}

impl Frontend {
    pub fn new<T: 'static>(backend: T) -> Frontend
        where T: Backend
    {
        Frontend {
            backend: Box::new(backend),
            queue: CommandQueue::new(),
        }
    }

    pub fn load_render_path(&mut self) {
        unimplemented!();
    }

    /// Draws a frame with the currently set render path. TODO: Build actual
    /// modular, parallelized Object translators.
    pub fn draw(&mut self, frame: Frame) {
        for light in frame.lights {
            self.queue.submit(match light {
                Light::Area => CommandEncoder::new().finish(),
                Light::Directional { color, direction, intensity } => {
                    CommandEncoder::new().finish()
                }
                Light::Point { color, intensity, location } => {
                    CommandEncoder::new().finish()
                }
                Light::Spot { angle, color, direction, intensity, location } => {
                    CommandEncoder::new().finish()
                }
            });
        }

        for obj in frame.objects {
            self.queue.submit(match obj {
                Object::Emitter => CommandEncoder::new().finish(),
                Object::IndexedMesh { indices, vertices } => {
                    CommandEncoder::new()
                        .set_buffer(indices)
                        .set_buffer(vertices)
                        .draw_indexed(0, 0, 0)
                        .finish()
                }
                Object::Mesh { vertices } => {
                    CommandEncoder::new()
                        .set_buffer(vertices)
                        .draw(0, 0)
                        .finish()
                }
                Object::Sprite => {
                    CommandEncoder::new()
                        .draw(0, 3)
                        .finish()
                }
            });
        }

        let commands = self.queue.sort_and_flush();
        self.backend.process(commands);
    }
}
