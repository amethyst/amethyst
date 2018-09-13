use amethyst_core::cgmath::{Point3, Vector3};
use amethyst_core::specs::{Component, DenseVecStorage};
use color::Rgba;
use vertex::PosColorNorm;

/// Debug lines are stored as a start position, a direction and a color.
///
/// Storing a direction instead of an end position may not be intuitive,
/// but is similar to other 'VertexFormat's.
pub type DebugLine = PosColorNorm;

/// Resource that stores debug lines to be rendered in DebugLinesPass draw pass
#[derive(Debug, Default)]
pub struct DebugLines {
    /// Lines to be rendered
    pub lines: Vec<DebugLine>,
}

impl Component for DebugLines {
    type Storage = DenseVecStorage<Self>;
}

impl DebugLines {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new() -> DebugLines {
        DebugLines {
            lines: Vec::<DebugLine>::new(),
        }
    }

    /// Builder method to pre-allocate a number of line.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<DebugLine>::with_capacity(capacity);
        self
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_as_direction(
        &mut self,
        position: Point3<f32>,
        direction: Vector3<f32>,
        color: Rgba,
    ) {
        let vertex = DebugLine {
            position: position.into(),
            color: color.into(),
            normal: direction.into(),
        };

        self.lines.push(vertex);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_as_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgba) {
        let vertex = DebugLine {
            position: start.into(),
            color: color.into(),
            normal: (end - start).into(),
        };

        self.lines.push(vertex);
    }
}
