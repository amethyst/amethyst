//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
    math::{Matrix4, Orthographic3, Perspective3},
};
use amethyst_error::Error;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Orthographic {
    matrix: Matrix4<f32>,
}
impl Orthographic {
    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut matrix = Matrix4::<f32>::identity();
        matrix[(0, 0)] = 2.0 / (right - left);
        matrix[(1, 1)] = 2.0 / (top - bottom);
        matrix[(2, 2)] = -1.0 / (z_far - z_near);
        matrix[(0, 3)] = -(right + left) / (right - left);
        matrix[(1, 3)] = -(top + bottom) / (top - bottom);
        matrix[(2, 3)] = -z_near / (z_far - z_near);
        Self { matrix }
    }

    #[inline]
    pub fn top(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn left(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn right(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn near(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn far(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn set_top(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_bottom(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_left(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_right(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn as_matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    #[inline]
    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.matrix
    }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Perspective {
    matrix: Matrix4<f32>,
}
impl Perspective {
    pub fn new(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Self {
        // Important: nalgebra's methods on Perspective3 are not safe for use with RH matrices
        let mut matrix = Matrix4::<f32>::identity();
        let tan_half_fovy = (fov / 2.0).tan();

        matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        matrix[(1, 1)] = -1.0 / tan_half_fovy;
        matrix[(2, 2)] = z_far / (z_near - z_far);
        matrix[(2, 3)] = -(z_near * z_far) / (z_far - z_near);
        matrix[(3, 2)] = -1.0;
        matrix[(3, 3)] = 0.0;

        Self {
            matrix,
        }
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        (self.matrix[(1, 1)] / self.matrix[(0, 0)]).abs()
    }

    #[inline]
    pub fn fov(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn fovx(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn fovy(&self) -> f32 {
        (-1.0 / self.matrix[(1, 1)]).atan() * 2.0
    }

    #[inline]
    pub fn near(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn far(&self) -> f32 {
        unimplemented!()
    }

    #[inline]
    pub fn set_aspect(&mut self, aspect: f32) {
        self.matrix[(0, 0)] = (self.matrix[(1, 1)] / aspect) * -1.0;
    }

    #[inline]
    pub fn set_fov(&mut self, fov: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_near(&mut self, near: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn set_far(&mut self, far: f32) {
        unimplemented!()
    }

    #[inline]
    pub fn as_matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    #[inline]
    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.matrix
    }
}


/// The projection mode of a `Camera`.
///
/// TODO: Remove and integrate with `Camera`.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub enum Projection {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    Orthographic(Orthographic),
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective(Perspective),

    Matrix(Matrix4<f32>),
}

impl Projection {
    /// Creates an orthographic projection with the given left, right, bottom, and
    /// top plane distances.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
    ) -> Projection {
        Projection::Orthographic(Orthographic::new(left, right, bottom, top, z_near, z_far))
    }

    /// Creates a perspective projection with the given aspect ratio and
    /// field-of-view. `fov` is specified in radians.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn perspective(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Projection {
        Projection::Perspective(Perspective::new(aspect, fov, z_near, z_far))
    }

    pub fn as_orthographic(&self) -> &Orthographic {
        match self {
            Projection::Orthographic(ref s) => s,
            _ => panic!("Failed to retrieve perspective"),
        }
    }

    pub fn as_perspective(&self) -> &Perspective {
        match self {
            Projection::Perspective(ref s) => s,
            _ => panic!("Failed to retrieve perspective"),
        }
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match self {
            Projection::Orthographic(ref s) => s.as_matrix(),
            Projection::Perspective(ref s) => s.as_matrix(),
            Projection::Matrix(ref s) => s,

        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match self {
            Projection::Orthographic(ref mut s) => s.as_matrix_mut(),
            Projection::Perspective(ref mut s) => s.as_matrix_mut(),
            Projection::Matrix(ref mut s) => s,

        }
    }
}

impl From<Orthographic> for Projection {
    fn from(proj: Orthographic) -> Self {
        Projection::Orthographic(proj)
    }
}

impl From<Perspective> for Projection {
    fn from(proj: Perspective) -> Self {
        Projection::Perspective(proj)
    }
}

impl From<Matrix4<f32>> for Projection {
    fn from(proj: Matrix4<f32>) -> Self {
        Projection::Matrix(proj)
    }
}

impl From<Projection> for Camera {
    fn from(proj: Projection) -> Self {
        Camera { inner: proj }
    }
}

/// Camera struct.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub struct Camera {
    /// Graphical projection of the camera.
    inner: Projection,
}

impl Camera {
    /// Create a normalized camera for 2D.
    ///
    /// Will use an orthographic projection centered around (0, 0) of size (width, height)
    /// Bottom left corner is (-width/2.0, -height/2.0)
    /// View transformation will be multiplicative identity.
    pub fn standard_2d(width: f32, height: f32) -> Self {
        // TODO: Check if bottom = height/2.0 is really the solution we want here.
        // Maybe the same problem as with the perspective matrix.
        Self::from(Projection::orthographic(
            -width / 2.0,
            width / 2.0,
            height / 2.0,
            -height / 2.0,
            0.1,
            2000.0,
        ))
    }

    /// Create a standard camera for 3D.
    ///
    /// Will use a perspective projection with aspect from the given screen dimensions and a field
    /// of view of π/3 radians (60 degrees).
    /// View transformation will be multiplicative identity.
    pub fn standard_3d(width: f32, height: f32) -> Self {
        Self::from(Projection::perspective(
            width / height,
            std::f32::consts::FRAC_PI_3,
            0.1,
            2000.0,
        ))
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match self.inner {
            Projection::Matrix(ref m) => m,
            Projection::Orthographic(ref p) => p.as_matrix(),
            Projection::Perspective(ref p) => p.as_matrix(),
        }
    }

    pub fn as_matrix_mut(&mut self) -> &mut Matrix4<f32> {
        match self.inner {
            Projection::Matrix(ref mut m) => m,
            Projection::Orthographic(ref mut p) => p.as_matrix_mut(),
            Projection::Perspective(ref mut p) => p.as_matrix_mut(),
        }
    }

    pub fn projection(&self) -> &Projection {
        &self.inner
    }

    pub fn set_projection(&mut self, new: Projection) {
        self.inner = new;
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
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CameraPrefab(Projection);

impl<'a> PrefabData<'a> for CameraPrefab {
    type SystemData = WriteStorage<'a, Camera>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage.insert(entity, Camera { inner: self.0.clone() })?;
        Ok(())
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
        _: &[Entity],
    ) -> Result<(), Error> {
        if let Some(ref mut cam) = system_data.0 {
            cam.entity = entities[self.0];
        }
        // TODO: if no `ActiveCamera` insert using `LazyUpdate`, require changes to `specs`
        Ok(())
    }
}
