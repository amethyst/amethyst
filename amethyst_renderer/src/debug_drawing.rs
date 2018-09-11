use amethyst_core::specs::{
    Component, DenseVecStorage, Entity, HashMapStorage, Write, WriteStorage,
};
use color::Rgba;
use vertex::PosColorNorm;

/// Resource that stores debug lines to be rendered in DebugLinesPass draw pass
#[derive(Debug)]
pub struct DebugLines {
    /// Lines to be rendered this frame
    pub lines: Vec<PosColorNorm>,
}

impl DebugLines {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new() -> DebugLines {
        DebugLines {
            lines: Vec::<PosColorNorm>::new(),
        }
    }

    /// Builder method to pre-allocate a number of line.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<PosColorNorm>::with_capacity(capacity);
        self
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_as_direction(&mut self, position: [f32; 3], direction: [f32; 3], color: Rgba) {
        let vertex = PosColorNorm {
            position: position,
            color: color.into(),
            normal: direction,
        };

        self.lines.push(vertex);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_as_line(&mut self, start: [f32; 3], end: [f32; 3], color: Rgba) {
        let vertex = PosColorNorm {
            position: start,
            color: color.into(),
            normal: [end[0] - start[0], end[1] - start[1], end[2] - start[2]],
        };

        self.lines.push(vertex);
    }
}

impl Component for DebugLines {
    type Storage = DenseVecStorage<Self>;
}
