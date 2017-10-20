//! Camera type with support for perspective and orthographic projections.

use cgmath::{Deg, Matrix4, Ortho, PerspectiveFov, Point3, Vector3};

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
///
/// TODO: Add more convenience methods, refine API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Camera {
    /// Location of the camera in three-dimensional space.
    pub eye: Point3<f32>,
    /// Graphical projection of the camera.
    pub proj: Matrix4<f32>,
    /// Forward vector of the camera.
    pub forward: Vector3<f32>,
    /// Right vector of the camera.
    pub right: Vector3<f32>,
    /// Upward elevation vector of the camera.
    pub up: Vector3<f32>,
}

impl Camera {
    /// Create a camera from the given projection, and with the view matrix as multiplicative identity.
    pub fn with_identity_view<P>(proj: P) -> Self
    where
        P: Into<Matrix4<f32>>,
    {
        use cgmath::EuclideanSpace;
        Self {
            eye: Point3::origin(),
            proj: proj.into(),
            forward: -Vector3::unit_z(),
            right: Vector3::unit_x(),
            up: Vector3::unit_y(),
        }
    }

    /// Create a normalized camera for 2D.
    ///
    /// Will use an orthographic projection with lower left corner being (-1., -1.) and
    /// upper right (1., 1.).
    /// View transformation will be multiplicative identity.
    pub fn standard_2d() -> Self {
        Self::with_identity_view(Projection::orthographic(-1., 1., 1., -1.))
    }

    /// Create a standard camera for 3D.
    ///
    /// Will use a perspective projection with aspect from the given screen dimensions and a field
    /// of view of 60 degrees.
    /// View transformation will be multiplicative identity.
    pub fn standard_3d(width: f32, height: f32) -> Self {
        use cgmath::Deg;
        Self::with_identity_view(Projection::perspective(width / height, Deg(60.)))
    }

    /// Calculates the view matrix from the given data.
    pub fn to_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.eye, self.eye + self.forward, self.up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_cam() {
        use cgmath::SquareMatrix;
        let cam = Camera::standard_2d();
        assert!(cam.to_view_matrix().is_identity());
    }
}
