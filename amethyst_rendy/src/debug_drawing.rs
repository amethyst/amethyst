//! Debug Drawing library
use amethyst_core::math::{Point2, Point3, UnitQuaternion, Vector2, Vector3};
use palette::Srgba;
use rendy::mesh::{AsVertex, Color, PosColor, VertexFormat};

use crate::pod::IntoPod;

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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

    /// Adds multiple lines that form a rectangle to be rendered by giving a Z coordinate, a min and a max position.
    ///
    /// This rectangle is aligned to the XY plane.
    pub fn add_rectangle_2d(&mut self, min: Point2<f32>, max: Point2<f32>, z: f32, color: Srgba) {
        self.add_line(
            [min[0], min[1], z].into(),
            [max[0], min[1], z].into(),
            color,
        );
        self.add_line(
            [min[0], min[1], z].into(),
            [min[0], max[1], z].into(),
            color,
        );
        self.add_line(
            [max[0], min[1], z].into(),
            [max[0], max[1], z].into(),
            color,
        );
        self.add_line(
            [min[0], max[1], z].into(),
            [max[0], max[1], z].into(),
            color,
        );
    }

    /// Adds multiple lines that form a rotated rectangle to be rendered by giving a Z coordinate, a rotation, a min and a max position.
    pub fn add_rotated_rectangle(
        &mut self,
        min: Point2<f32>,
        max: Point2<f32>,
        z: f32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        let center = (min + Vector2::new(max[0], max[1])) / 2.0;
        let center = Vector3::new(center[0], center[1], z);

        let top_left = Point3::new(min[0], min[1], z);
        let top_right = Point3::new(min[0], max[1], z);
        let bottom_left = Point3::new(max[0], min[1], z);
        let bottom_right = Point3::new(max[0], max[1], z);

        let top_left = rotation * (top_left - center) + center;
        let top_right = rotation * (top_right - center) + center;
        let bottom_left = rotation * (bottom_left - center) + center;
        let bottom_right = rotation * (bottom_right - center) + center;

        self.add_line(top_left, top_right, color);
        self.add_line(top_left, bottom_left, color);
        self.add_line(top_right, bottom_right, color);
        self.add_line(bottom_left, bottom_right, color);
    }

    /// Adds multiple lines that form a box to be rendered by giving a min and a max position.
    ///
    /// This box is an axis aligned box.
    pub fn add_box(&mut self, min: Point3<f32>, max: Point3<f32>, color: Srgba) {
        self.add_rectangle_2d(min.xy(), max.xy(), min[2], color);
        self.add_rectangle_2d(min.xy(), max.xy(), max[2], color);
        self.add_line(min, [min[0], min[1], max[2]].into(), color);
        self.add_line(
            [max[0], min[1], min[2]].into(),
            [max[0], min[1], max[2]].into(),
            color,
        );
        self.add_line(
            [min[0], max[1], min[2]].into(),
            [min[0], max[1], max[2]].into(),
            color,
        );
        self.add_line([max[0], max[1], min[2]].into(), max, color);
    }

    /// Adds multiple lines that form a rotated box to be rendered by giving a rotation, a min and a max position.
    pub fn add_rotated_box(
        &mut self,
        min: Point3<f32>,
        max: Point3<f32>,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        let center = (min + Vector3::from([max[0], max[1], max[2]])) / 2.0;
        let center = Vector3::new(center[0], center[1], center[2]);

        let top_left_back = Point3::new(min[0], min[1], min[2]);
        let top_right_back = Point3::new(min[0], max[1], min[2]);
        let bottom_left_back = Point3::new(max[0], min[1], min[2]);
        let bottom_right_back = Point3::new(max[0], max[1], min[2]);

        let top_left_back = rotation * (top_left_back - center) + center;
        let top_right_back = rotation * (top_right_back - center) + center;
        let bottom_left_back = rotation * (bottom_left_back - center) + center;
        let bottom_right_back = rotation * (bottom_right_back - center) + center;

        let top_left_front = Point3::new(min[0], min[1], max[2]);
        let top_right_front = Point3::new(min[0], max[1], max[2]);
        let bottom_left_front = Point3::new(max[0], min[1], max[2]);
        let bottom_right_front = Point3::new(max[0], max[1], max[2]);

        let top_left_front = rotation * (top_left_front - center) + center;
        let top_right_front = rotation * (top_right_front - center) + center;
        let bottom_left_front = rotation * (bottom_left_front - center) + center;
        let bottom_right_front = rotation * (bottom_right_front - center) + center;

        self.add_line(top_left_back, top_right_back, color);
        self.add_line(top_left_back, bottom_left_back, color);
        self.add_line(top_left_back, top_left_front, color);
        self.add_line(top_right_back, bottom_right_back, color);
        self.add_line(top_right_back, top_right_front, color);
        self.add_line(bottom_left_back, bottom_right_back, color);
        self.add_line(bottom_left_back, bottom_left_front, color);
        self.add_line(top_left_front, top_right_front, color);
        self.add_line(top_left_front, bottom_left_front, color);
        self.add_line(bottom_right_front, top_right_front, color);
        self.add_line(bottom_right_front, bottom_left_front, color);
        self.add_line(bottom_right_front, bottom_right_back, color);
    }

    /// Adds multiple lines that form a circle to be rendered by giving a center, a radius and an amount of points.
    ///
    /// This circle is aligned to the XY plane.
    pub fn add_circle_2d(&mut self, center: Point3<f32>, radius: f32, points: u32, color: Srgba) {
        let mut prev = None;

        for i in 0..=points {
            let a = std::f32::consts::PI * 2.0 / (points as f32) * (i as f32);
            let x = radius * a.cos();
            let y = radius * a.sin();
            let point = [center[0] + x, center[1] + y, center[2]].into();

            if let Some(prev) = prev {
                self.add_line(prev, point, color);
            }

            prev = Some(point);
        }
    }

    /// Adds multiple lines that form a rotated circle to be rendered by giving a center, a radius, an amount of points and a rotation.
    pub fn add_rotated_circle(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        points: u32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        let mut prev = None;

        for i in 0..=points {
            let a = std::f32::consts::PI * 2.0 / (points as f32) * (i as f32);
            let x = radius * a.cos();
            let y = radius * a.sin();
            let point = Vector3::new(x, y, 0.0);
            let point = center + rotation * point;

            if let Some(prev) = prev {
                self.add_line(prev, point, color);
            }

            prev = Some(point);
        }
    }

    /// Adds multiple lines that form a sphere to be rendered by giving a center, a radius, an amount of vertical points and an amount of horizontal points.
    pub fn add_sphere(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        horizontal_points: u32,
        vertical_points: u32,
        color: Srgba,
    ) {
        let mut prev_row = Vec::new();

        for i in 0..=horizontal_points {
            let lon = std::f32::consts::PI / (horizontal_points as f32) * (i as f32);
            let mut new_prev_row = Vec::new();

            for j in 0..=vertical_points {
                let lat = std::f32::consts::PI * 2.0 / (vertical_points as f32) * (j as f32);
                let x = radius * lon.sin() * lat.cos();
                let y = radius * lon.sin() * lat.sin();
                let z = radius * lon.cos();
                let point = center + Vector3::new(x, y, z);

                if !new_prev_row.is_empty() {
                    self.add_line(new_prev_row[(j - 1) as usize], point, color);
                }

                if !prev_row.is_empty() {
                    self.add_line(prev_row[j as usize], point, color);
                }

                new_prev_row.push(point);
            }

            prev_row = new_prev_row;
        }
    }

    /// Adds multiple lines that form a cylinder to be rendered by giving a center, a radius, a height and an amount of points.
    ///
    /// This cylinder is aligned to the y axis.
    pub fn add_cylinder(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        height: f32,
        points: u32,
        color: Srgba,
    ) {
        let mut prev: Option<(Point3<f32>, Point3<f32>)> = None;

        for i in 0..=points {
            let a = std::f32::consts::PI * 2.0 / (points as f32) * (i as f32);
            let x = radius * a.cos();
            let z = radius * a.sin();
            let point1: Point3<f32> =
                [center[0] + x, center[1] - height / 2.0, center[2] + z].into();
            let point2: Point3<f32> = [point1[0], point1[1] + height, point1[2]].into();

            self.add_line(point1, point2, color);

            if let Some(prev) = prev {
                self.add_line(prev.0, point1, color);
                self.add_line(prev.1, point2, color);
            }

            prev = Some((point1, point2));
        }
    }

    /// Adds multiple lines that form a rotated cylinder to be rendered by giving a center, a radius, a height, an amount of points and a rotation.
    pub fn add_rotated_cylinder(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        height: f32,
        points: u32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        let mut prev: Option<(Point3<f32>, Point3<f32>)> = None;
        let center = Vector3::new(center[0], center[1], center[2]);

        for i in 0..=points {
            let a = std::f32::consts::PI * 2.0 / (points as f32) * (i as f32);
            let x = radius * a.cos();
            let z = radius * a.sin();
            let point1: Point3<f32> =
                [center[0] + x, center[1] - height / 2.0, center[2] + z].into();
            let point2: Point3<f32> = [point1[0], point1[1] + height, point1[2]].into();
            let point1 = rotation * (point1 - center) + center;
            let point2 = rotation * (point2 - center) + center;

            self.add_line(point1, point2, color);

            if let Some(prev) = prev {
                self.add_line(prev.0, point1, color);
                self.add_line(prev.1, point2, color);
            }

            prev = Some((point1, point2));
        }
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

    /// Submits multiple lines that form a rectangle to be rendered by giving a Z coordinate, a min and a max position.
    ///
    /// This rectangle is aligned to the XY plane.
    pub fn draw_rectangle(&mut self, min: Point2<f32>, max: Point2<f32>, z: f32, color: Srgba) {
        self.inner.add_rectangle_2d(min, max, z, color);
    }

    /// Submits multiple lines that form a rotated rectangle to be rendered by giving a Z coordinate, a rotation, a min and a max position.
    pub fn draw_rotated_rectangle(
        &mut self,
        min: Point2<f32>,
        max: Point2<f32>,
        z: f32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        self.inner
            .add_rotated_rectangle(min, max, z, rotation, color);
    }

    /// Submits multiple lines that form a box to be rendered by giving a min and a max position.
    ///
    /// This box is an axis aligned box.
    pub fn draw_box(&mut self, min: Point3<f32>, max: Point3<f32>, color: Srgba) {
        self.inner.add_box(min, max, color);
    }

    /// Submits multiple lines that form a rotated box to be rendered by giving a rotation, a min and a max position.
    ///
    /// This box is an axis aligned box.
    pub fn draw_rotated_box(
        &mut self,
        min: Point3<f32>,
        max: Point3<f32>,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        self.inner.add_rotated_box(min, max, rotation, color);
    }

    /// Submits multiple lines that form a circle to be rendered by giving a center, a radius and an amount of points.
    ///
    /// This circle is aligned to the XY plane.
    pub fn draw_circle(&mut self, center: Point3<f32>, radius: f32, points: u32, color: Srgba) {
        self.inner.add_circle_2d(center, radius, points, color);
    }

    /// Submits multiple lines that form a rotated circle to be rendered by giving a center, a radius, an amount of points and a rotation.
    pub fn draw_rotated_circle(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        points: u32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        self.inner
            .add_rotated_circle(center, radius, points, rotation, color);
    }

    /// Submits multiple lines that form a sphere to be rendered by giving a center, a radius, an amount of vertical points and an amount of horizontal points.
    pub fn draw_sphere(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        horizontal_points: u32,
        vertical_points: u32,
        color: Srgba,
    ) {
        self.inner
            .add_sphere(center, radius, horizontal_points, vertical_points, color);
    }

    /// Submits multiple lines that form a cylinder to be rendered by giving a center, a radius, a height and an amount of points.
    ///
    /// This cylinder is aligned to the y axis.
    pub fn draw_cylinder(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        height: f32,
        points: u32,
        color: Srgba,
    ) {
        self.inner
            .add_cylinder(center, radius, height, points, color);
    }

    /// Adds multiple lines that form a rotated cylinder to be rendered by giving a center, a radius, a height, an amount of points and a rotation.
    pub fn draw_rotated_cylinder(
        &mut self,
        center: Point3<f32>,
        radius: f32,
        height: f32,
        points: u32,
        rotation: UnitQuaternion<f32>,
        color: Srgba,
    ) {
        self.inner
            .add_rotated_cylinder(center, radius, height, points, rotation, color);
    }

    pub(crate) fn drain(&mut self) -> impl Iterator<Item = DebugLine> + '_ {
        self.inner.lines.drain(..)
    }
}
