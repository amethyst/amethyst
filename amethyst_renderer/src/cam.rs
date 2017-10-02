//! Camera type with support for perspective and orthographic projections.

use cgmath::{Deg, EuclideanSpace, Euler, InnerSpace, Matrix4, Ortho, PerspectiveFov, Point3, Quaternion, Rotation, Vector3};

/// The projection mode of a `Camera`.
///
/// TODO: Remove and integrate with `Camera`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Projection {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    Orthographic(Ortho<f32>),
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective(PerspectiveFov<f32>),
}

impl Projection {
    /// Creates an orthographic projection with the given left, right, top, and
    /// bottom plane distances.
    pub fn orthographic(l: f32, r: f32, t: f32, b: f32) -> Projection {
        Projection::Orthographic(Ortho {
            left: l,
            right: r,
            top: t,
            bottom: b,
            near: 0.1,
            far: 2000.0,
        })
    }

    /// Creates a perspective projection with the given aspect ratio and
    /// field-of-view.
    pub fn perspective<D: Into<Deg<f32>>>(aspect: f32, fov: D) -> Projection {
        Projection::Perspective(PerspectiveFov {
            fovy: fov.into().into(),
            aspect: aspect,
            near: 0.1,
            far: 2000.0,
        })
    }
}

impl From<Projection> for Matrix4<f32> {
    fn from(proj: Projection) -> Self {
        match proj {
            Projection::Orthographic(ortho) => ortho.into(),
            Projection::Perspective(perspective) => perspective.into(),
        }
    }
}

/// Camera struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Camera {
    /// Initial forward vector of the camera.
    pub base_forward: Vector3<f32>,
    /// Calculated center point of the camera.
    center: Vector3<f32>,
    /// Trigger re-calculations if values were changed.
    pub dirty: bool,
    /// Location of the camera in three-dimensional space.
    pub eye: Point3<f32>,
    /// Graphical projection of the camera.
    pub proj: Matrix4<f32>,
    /// Rotation of the camera in three-dimensional space.
    pub rotation: Quaternion<f32>,
    /// Upward elevation vector of the camera.
    pub up: Vector3<f32>,
}

impl Camera {
    /// Create a new camera from the given values.
    pub fn new(
        projection: Matrix4<f32>,
        position: Point3<f32>,
        look_at: Point3<f32>,
        up: Vector3<f32>
    ) -> Self {
        if position == look_at {
            panic!("Position and LookAt cannot have the same coordinates!");
        }

        let base_forward = (look_at - position);
        Self {
            base_forward: base_forward,
            center: (position + base_forward).to_vec(),
            dirty: false,
            eye: position,
            proj: projection,
            rotation: Quaternion::look_at(base_forward, up).normalize(),
            up: up,
        }
    }

    /// Calculates the view matrix from the given data.
    pub fn to_view_matrix(&self) -> Matrix4<f32> {
        if self.dirty { panic!("The camera needs to be updated before being queried!"); }

        let center = Point3::new(self.center.x, self.center.y, self.center.z);
        Matrix4::look_at(self.eye, center, self.up)
    }

    /// Update center component of camera.
    pub fn update(&mut self) {
        self.center = (self.eye + self.rotation.rotate_vector(self.base_forward)).to_vec();
        self.dirty = false;
    }
}
