use amethyst_core::{
    nalgebra::{Point3, Vector3},
    specs::{Component, DenseVecStorage},
};

use crate::{color::Rgba, vertex::PosColorNorm};

/// Debug lines are stored as a position, a direction and a color.
///
/// Storing a direction instead of an end position may not be intuitive,
/// but is similar to other 'VertexFormat's.
pub type DebugLine = PosColorNorm;

/// Component that stores persistent debug lines to be rendered in DebugLinesPass draw pass.
/// The vector can only be cleared manually.
#[derive(Debug, Default)]
pub struct DebugLinesComponent {
    /// Lines to be rendered
    pub lines: Vec<DebugLine>,
}

impl Component for DebugLinesComponent {
    type Storage = DenseVecStorage<Self>;
}

impl DebugLinesComponent {
    /// Creates a new debug lines component with an empty DebugLine vector.
    pub fn new() -> DebugLinesComponent {
        DebugLinesComponent {
            lines: Vec::<DebugLine>::new(),
        }
    }

    /// Builder method to pre-allocate a number of lines.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<DebugLine>::with_capacity(capacity);
        self
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Rgba) {
        let vertex = DebugLine {
            position: position.to_homogeneous().xyz().into(),
            color: color.into(),
            normal: direction.into(),
        };

        self.lines.push(vertex);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgba) {
        let vertex = DebugLine {
            position: start.to_homogeneous().xyz().into(),
            color: color.into(),
            normal: (end - start).into(),
        };

        self.lines.push(vertex);
    }

    /// Clears lines buffer.
    ///
    /// As lines are persistent, it's necessary to use this function for updating or deleting lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Resource that stores non-persistent debug lines to be rendered in DebugLinesPass draw pass.
/// The vector is automatically cleared after being rendered.
#[derive(Debug, Default)]
pub struct DebugLines {
    /// Lines to be rendered
    pub lines: Vec<DebugLine>,
}

impl DebugLines {
    /// Creates a new debug lines component with an empty DebugLine vector.
    pub fn new() -> DebugLines {
        DebugLines {
            lines: Vec::<DebugLine>::new(),
        }
    }

    /// Builder method to pre-allocate a number of lines.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<DebugLine>::with_capacity(capacity);
        self
    }

    /// Submits a line to be rendered by giving a position and a direction.
    pub fn draw_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Rgba) {
        let vertex = DebugLine {
            position: position.to_homogeneous().xyz().into(),
            color: color.into(),
            normal: direction.into(),
        };

        self.lines.push(vertex);
    }

    /// Submits a line to be rendered by giving a start and an end position.
    pub fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Rgba) {
        let vertex = DebugLine {
            position: start.to_homogeneous().xyz().into(),
            color: color.into(),
            normal: (end - start).into(),
        };

        self.lines.push(vertex);
    }
}
