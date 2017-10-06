//! Camera type with support for perspective and orthographic projections.

use amethyst::ecs::transform::LocalTransform;
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
    pub proj: Matrix4<f32>,
}

impl Camera {
    /// Create a new camera from the given values.
    pub fn new(projection: Matrix4<f32>) -> Self {
        Self {
            proj: projection,
        }
    }
}
