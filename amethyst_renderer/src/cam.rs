//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::{
    cgmath::{Deg, Matrix4, Ortho, PerspectiveFov},
    specs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
};

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
            aspect,
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

impl From<Projection> for Camera {
    fn from(proj: Projection) -> Self {
        Self { proj: proj.into() }
    }
}

/// Camera struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Camera {
    /// Graphical projection of the camera.
    pub proj: Matrix4<f32>,
}

impl Camera {
    /// Create a normalized camera for 2D.
    ///
    /// Will use an orthographic projection with lower left corner being (-1., -1.) and
    /// upper right (1., 1.).
    /// View transformation will be multiplicative identity.
    pub fn standard_2d() -> Self {
        Self::from(Projection::orthographic(-1., 1., 1., -1.))
    }

    /// Create a standard camera for 3D.
    ///
    /// Will use a perspective projection with aspect from the given screen dimensions and a field
    /// of view of 60 degrees.
    /// View transformation will be multiplicative identity.
    pub fn standard_3d(width: f32, height: f32) -> Self {
        use amethyst_core::cgmath::Deg;
        Self::from(Projection::perspective(width / height, Deg(60.)))
    }
}

impl Component for Camera {
    type Storage = HashMapStorage<Self>;
}

/// Active camera resource, used by the renderer to choose which camera to get the view matrix from.
/// If no active camera is found, the first camera will be used as a fallback.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveCamera {
    /// Camera entity
    pub entity: Entity,
}

/// Projection prefab
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CameraPrefab {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    Orthographic(Ortho<f32>),

    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective(PerspectiveFov<f32>),

    /// Projection matrix
    Matrix(Matrix4<f32>),
}

impl<'a> PrefabData<'a> for CameraPrefab {
    type SystemData = WriteStorage<'a, Camera>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let proj = match *self {
            CameraPrefab::Matrix(mat) => mat,
            CameraPrefab::Orthographic(ortho) => ortho.into(),
            CameraPrefab::Perspective(perspective) => perspective.into(),
        };
        storage.insert(entity, Camera { proj }).map(|_| ())
    }
}

/// Active camera prefab
pub struct ActiveCameraPrefab(usize);

impl<'a> PrefabData<'a> for ActiveCameraPrefab {
    type SystemData = (Option<Write<'a, ActiveCamera>>,);
    type Result = ();

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        if let Some(ref mut cam) = system_data.0 {
            cam.entity = entities[self.0];
        }
        // TODO: if no `ActiveCamera` insert using `LazyUpdate`, require changes to `specs`
        Ok(())
    }
}
