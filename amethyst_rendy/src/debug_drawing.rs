use crate::pod::IntoPod;
use amethyst_core::{
    ecs::{Component, DenseVecStorage},
    math::{Point3, Vector3},
};
use palette::Srgba;
use rendy::mesh::{AsVertex, Color, PosColor, VertexFormat};

/// Debug lines are stored as a pair of position and color.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct DebugLine {
    start: PosColor,
    end: PosColor,
}

impl AsVertex for DebugLine {
    fn vertex() -> VertexFormat {
        VertexFormat::new((PosColor::vertex(), PosColor::vertex()))
    }
}

impl DebugLine {
    fn new(start: PosColor, end: PosColor) -> Self {
        Self { start, end }
    }
}

/// Parameters for renderer of debug lines. The params affect all lines.
pub struct DebugLinesParams {
    /// Width of lines in screen space pixels, default is 1.0 pixel
    pub line_width: f32,
}

impl Default for DebugLinesParams {
    fn default() -> Self {
        DebugLinesParams { line_width: 1.0 }
    }
}

/// Component that stores persistent debug lines to be rendered in DebugLinesPass draw pass.
/// The vector can only be cleared manually.
#[derive(Debug, Default)]
pub struct DebugLinesComponent {
    /// Lines to be rendered
    lines: Vec<DebugLine>,
}

impl Component for DebugLinesComponent {
    type Storage = DenseVecStorage<Self>;
}

impl DebugLinesComponent {
    /// Creates a new debug lines component with an empty DebugLine vector.
    pub fn new() -> DebugLinesComponent {
        Self::default()
    }

    /// Builder method to pre-allocate a number of lines.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: Vec::with_capacity(capacity),
        }
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Srgba) {
        self.add_line(position, position + direction, color);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Srgba) {
        self.add_gradient_line(start, end, color, color);
    }

    /// Adds a line to be rendered by giving a start and an end position with separate start and end colors.
    pub fn add_gradient_line(
        &mut self,
        start: Point3<f32>,
        end: Point3<f32>,
        start_color: Srgba,
        end_color: Srgba,
    ) {
        let vertex = DebugLine::new(
            PosColor {
                position: start.to_homogeneous().xyz().into(),
                color: Color(start_color.into_pod()),
            },
            PosColor {
                position: end.to_homogeneous().xyz().into(),
                color: Color(end_color.into_pod()),
            },
        );
        self.lines.push(vertex);
    }

    /// Clears lines buffer.
    ///
    /// As lines are persistent, it's necessary to use this function for updating or deleting lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub(crate) fn lines(&self) -> &[DebugLine] {
        &self.lines
    }
}

/// Resource that stores non-persistent debug lines to be rendered in DebugLinesPass draw pass.
/// The vector is automatically cleared after being rendered.
#[derive(Debug, Default)]
pub struct DebugLines {
    /// Lines to be rendered
    inner: DebugLinesComponent,
}

impl DebugLines {
    /// Creates a new debug lines component with an empty DebugLine vector.
    pub fn new() -> DebugLines {
        Self {
            inner: Default::default(),
        }
    }

    /// Submits a line to be rendered by giving a position and a direction.
    pub fn draw_direction(&mut self, position: Point3<f32>, direction: Vector3<f32>, color: Srgba) {
        self.inner.add_direction(position, direction, color);
    }

    /// Submits a line to be rendered by giving a start and an end position with separate start and end colors.
    pub fn draw_gradient_line(
        &mut self,
        start: Point3<f32>,
        end: Point3<f32>,
        start_color: Srgba,
        end_color: Srgba,
    ) {
        self.inner
            .add_gradient_line(start, end, start_color, end_color);
    }

    /// Submits a line to be rendered by giving a start and an end position.
    pub fn draw_line(&mut self, start: Point3<f32>, end: Point3<f32>, color: Srgba) {
        self.inner.add_line(start, end, color);
    }

    pub(crate) fn drain<'a>(&'a mut self) -> impl Iterator<Item = DebugLine> + 'a {
        self.inner.lines.drain(..)
    }
}
