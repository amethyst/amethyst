use amethyst_core::{
    ecs::{Component, DenseVecStorage}
};

use rendy::mesh::{Position, Color};

#[derive(Debug, Clone)]
pub struct DebugLine {
    /// Starting point of the line
    pub start: Position,
    /// RGBA color value of the line.
    pub color: Color,
    /// Endpoint of the line
    pub end: Position,
}

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

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_line(&mut self, start: Position, end: Position, color: Color) {
        let vertex = DebugLine {
            start,
            color,
            end
        };

        self.lines.push(vertex);
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

    /// Submits a line to be rendered by giving a start and an end position.
    pub fn draw_line(&mut self, start: Position, end: Position, color: Color) {
        let vertex = DebugLine {
            start,
            color,
            end
        };

        self.lines.push(vertex);
    }
}
