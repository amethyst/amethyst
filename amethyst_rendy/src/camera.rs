//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
    math::{Matrix4, Orthographic3, Perspective3},
};
use amethyst_error::Error;

/// The projection mode of a `Camera`.
///
/// TODO: Remove and integrate with `Camera`.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub enum Projection {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    Orthographic(Orthographic3<f32>),
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective(Perspective3<f32>),
}

impl Projection {
    /// Creates an orthographic projection with the given left, right, bottom, and
    /// top plane distances.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32,) -> Projection {
        let mut proj = Matrix4::<f32>::identity();

        proj[(0, 0)] = 2.0 / (right - left);
        proj[(1, 1)] = 2.0 / (top - bottom);
        proj[(2, 2)] = - 1.0 / (z_far - z_near);
        proj[(0, 3)] = - (right + left) / (right - left);
        proj[(1, 3)] = - (top + bottom) / (top - bottom);
        proj[(2, 3)] = - z_near / (z_far  - z_near);

        // Important: nalgebra's methods on Orthographic3 are not safe for use with RH matrices
        Projection::Orthographic(Orthographic3::from_matrix_unchecked(proj))
    }

    /// Creates a perspective projection with the given aspect ratio and
    /// field-of-view. `fov` is specified in radians.
    /// The projection matrix is right-handed and has a depth range of 0 to 1
    pub fn perspective(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Projection {
        let mut proj = Matrix4::<f32>::identity();
        
        let tan_half_fovy = (fov / 2.0).tan();

        proj[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        proj[(1, 1)] = - 1.0 / tan_half_fovy;
        proj[(2, 2)] = z_far / (z_near - z_far);

        proj[(2, 3)] = - (z_near * z_far) / (z_far - z_near);
        proj[(3, 2)] = - 1.0;
        proj[(3, 3)] = 0.0;

        // Important: nalgebra's methods on Perspective3 are not safe for use with RH matrices
        Projection::Perspective(Perspective3::from_matrix_unchecked(proj))
    }
}

impl From<Projection> for Camera {
    fn from(proj: Projection) -> Self {
        let proj = match proj {
            Projection::Perspective(p) => p.to_homogeneous(),
            Projection::Orthographic(o) => o.to_homogeneous(),
        };
        Camera { proj }
    }
}

/// Camera struct.
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
pub struct Camera {
    /// Graphical projection of the camera.
    pub proj: Matrix4<f32>,
}

impl Camera {
    /// Create a normalized camera for 2D.
    ///
    /// Will use an orthographic projection with top left corner being (-1., -1.) and
    /// lower right (1., 1.).
    /// View transformation will be multiplicative identity.
    pub fn standard_2d() -> Self {
        Self::from(Projection::orthographic(-1., 1., -1., 1., 0.1, 2000.0))
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
pub enum CameraPrefab {
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    #[serde(with = "serde_ortho")]
    Orthographic(Orthographic3<f32>),

    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    #[serde(with = "serde_persp")]
    Perspective(Perspective3<f32>),

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
        _: &[Entity],
    ) -> Result<(), Error> {
        let proj = match *self {
            CameraPrefab::Matrix(mat) => mat,
            CameraPrefab::Orthographic(ortho) => ortho.to_homogeneous(),
            CameraPrefab::Perspective(perspective) => perspective.to_homogeneous(),
        };
        storage.insert(entity, Camera { proj })?;
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

mod serde_ortho {
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    use amethyst_core::math::Orthographic3;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Orthographic3<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Orthographic::deserialize(deserializer)?;
        Ok(Orthographic3::new(
            values.left,
            values.right,
            values.bottom,
            values.top,
            values.znear,
            values.zfar,
        ))
    }

    pub fn serialize<S>(proj: &Orthographic3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &Orthographic {
                left: proj.left(),
                right: proj.right(),
                bottom: proj.bottom(),
                top: proj.top(),
                znear: proj.znear(),
                zfar: proj.zfar(),
            },
            serializer,
        )
    }
}

mod serde_persp {
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    use amethyst_core::math::Perspective3;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Perspective {
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Perspective3<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Perspective::deserialize(deserializer)?;
        Ok(Perspective3::new(
            values.aspect,
            values.fovy,
            values.znear,
            values.zfar,
        ))
    }

    pub fn serialize<S>(proj: &Perspective3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &Perspective {
                aspect: proj.aspect(),
                fovy: proj.fovy(),
                znear: proj.znear(),
                zfar: proj.zfar(),
            },
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amethyst_core::math;

    static ASPECT_RATIO : f32 = 16.0/9.0;


    #[test]
    fn standard_2d() {
        let cam = Camera::standard_2d();
        
    }

    #[test]
    fn standard_3d() {
        let cam = Camera::standard_3d(1280.0, 720.0);

    }

    #[test]
    fn orthographic_orientation() {
        let cam : Camera = Projection::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0).into();
        let vec_in1 = math::Vector4::<f32>::new(0.0, 0.0, -1.0, 1.0);
        let vec_out1 = cam.proj * vec_in1;
        let vec_out1_ref = math::Vector4::<f32>::new(0.0, 0.0, -1.0 / (100.0 - 0.1) * vec_in1[2] - 0.1 / (100.0 - 0.1), 1.0);
        assert_eq!(vec_out1_ref, vec_out1);

        let vec_in2 = math::Vector4::<f32>::new(0.5, 0.5, -99.0, 1.0);
        let vec_out2 = cam.proj * vec_in2;
        let z_c = -1.0 / (100.0 - 0.1) * vec_in2[2] - 0.1 / (100.0 - 0.1);

        let vec_out2_ref = math::Vector4::<f32>::new(0.5, 0.5, z_c, 1.0);
        assert_eq!(vec_out2_ref, vec_out2);

    }

    #[test]
    fn orthographic_depth() {
        let cam : Camera = Projection::orthographic(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0).into();
        let vec_in1 = math::Vector4::<f32>::new(0.0, -5.0, -0.1, 1.0);
        let vec_out1 = cam.proj * vec_in1;
        // let vec_out1_ref = math::Vector4::<f32>::new(0.0, -0.5, 0.002002002, 1.0);

        // let vec_in2 = math::Vector4::<f32>::new(0.0, -2.0, -50.1, 1.0);
        // let vec_out2 = cam.proj * vec_in2;
        // let vec_out2_ref = math::Vector4::<f32>::new(0.0, 0.4, 50.1, 1.0);

        // let vec_in3 = math::Vector4::<f32>::new(0.0, 2.0, -99.0, 1.0);
        // let vec_out3 = cam.proj * vec_in3;
        // let vec_out3_ref = math::Vector4::<f32>::new(0.0, -0.4, 99.0, 1.0);

        // let vec_in4 = math::Vector4::<f32>::new(0.0, 5.0, -100.1, 1.0);
        // let vec_out4 = cam.proj * vec_in4;
        // let vec_out4_ref = math::Vector4::<f32>::new(0.0, 0.4, 100.1, 1.0);

        // assert_eq!(vec_out1_ref, vec_out1);
        // assert_eq!(vec_out2_ref, vec_out2);
        // assert_eq!(vec_out3_ref, vec_out3);
        // assert_eq!(vec_out4_ref, vec_out4);
    }

    #[test]
    fn perspective_orientation() {
        let cam : Camera = Projection::perspective(ASPECT_RATIO, std::f32::consts::FRAC_PI_3, 0.1, 100.0).into();
        let vec_in1 = math::Vector4::<f32>::new(0.0, 0.0, -0.11, 1.0);
        let vec_out1 = cam.proj * vec_in1;
        assert!(vec_out1[2] >= 0.0);
        assert!(vec_out1[2] <= 1.0);

        let vec_in2 = math::Vector4::<f32>::new(0.0, 1.0, -99.0, 1.0);
        let vec_out2 = cam.proj * vec_in2;
        dbg!(vec_out2);
        assert!(vec_out2[2] >= 0.0);
        assert!(vec_out2[2] <= 1.0);
    }

    #[test]
    fn perspective_depth() {
        let cam : Camera = Projection::perspective(ASPECT_RATIO, std::f32::consts::FRAC_PI_3, 0.1 as f32, 2000.0).into();
    }
}