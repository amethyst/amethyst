//! Camera type with support for perspective and orthographic projections.

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::{
    nalgebra::{Matrix4, Orthographic3, Perspective3},
    specs::prelude::{Component, Entity, HashMapStorage, Write, WriteStorage},
};

use serde::{Deserialize, Serialize};

/// The projection mode of a `Camera`.
///
/// TODO: Remove and integrate with `Camera`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    pub fn orthographic(l: f32, r: f32, b: f32, t: f32) -> Projection {
        Projection::Orthographic(Orthographic3::new(l, r, b, t, 0.1, 2000.0))
    }

    /// Creates a perspective projection with the given aspect ratio and
    /// field-of-view. `fov` is specified in radians.
    pub fn perspective(aspect: f32, fov: f32) -> Projection {
        Projection::Perspective(Perspective3::new(aspect, fov, 0.1, 2000.0))
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
        Self::from(Projection::orthographic(-1., 1., -1., 1.))
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
        ))
    }
}

impl Component for Camera {
    type Storage = HashMapStorage<Self>;
}

/// Active camera resource, used by the renderer to choose which camera to get the view matrix from.
/// If no active camera is found, the first camera will be used as a fallback.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActiveCamera {
    /// Camera entity
    pub entity: Option<Entity>,
}

/// Projection prefab
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    ) -> Result<(), PrefabError> {
        let proj = match *self {
            CameraPrefab::Matrix(mat) => mat,
            CameraPrefab::Orthographic(ortho) => ortho.to_homogeneous(),
            CameraPrefab::Perspective(perspective) => perspective.to_homogeneous(),
        };
        storage.insert(entity, Camera { proj }).map(|_| ())
    }
}

/// Active camera prefab
pub struct ActiveCameraPrefab(usize);

impl<'a> PrefabData<'a> for ActiveCameraPrefab {
    type SystemData = (Write<'a, ActiveCamera>,);
    type Result = ();

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        system_data.0.entity = Some(entities[self.0]);
        // TODO: if no `ActiveCamera` insert using `LazyUpdate`, require changes to `specs`
        Ok(())
    }
}

mod serde_ortho {
    use std::fmt;

    use serde::{
        de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
        ser::Serializer,
        Deserialize, Serialize,
    };

    use amethyst_core::nalgebra::Orthographic3;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Orthographic3<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Left,
            Right,
            Bottom,
            Top,
            Znear,
            Zfar,
        };

        struct OrthographicVisitor;

        impl<'de> Visitor<'de> for OrthographicVisitor {
            type Value = Orthographic3<f32>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Orthographic")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let left = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let right = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let bottom = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let top = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let znear = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let zfar = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                Ok(Orthographic3::new(left, right, bottom, top, znear, zfar))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut left = None;
                let mut right = None;
                let mut bottom = None;
                let mut top = None;
                let mut znear = None;
                let mut zfar = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Left => {
                            if left.is_some() {
                                return Err(de::Error::duplicate_field("left"));
                            }
                            left = Some(map.next_value()?);
                        }
                        Field::Right => {
                            if right.is_some() {
                                return Err(de::Error::duplicate_field("right"));
                            }
                            right = Some(map.next_value()?);
                        }
                        Field::Bottom => {
                            if bottom.is_some() {
                                return Err(de::Error::duplicate_field("bottom"));
                            }
                            bottom = Some(map.next_value()?);
                        }
                        Field::Top => {
                            if top.is_some() {
                                return Err(de::Error::duplicate_field("top"));
                            }
                            top = Some(map.next_value()?);
                        }
                        Field::Znear => {
                            if znear.is_some() {
                                return Err(de::Error::duplicate_field("znear"));
                            }
                            znear = Some(map.next_value()?);
                        }
                        Field::Zfar => {
                            if zfar.is_some() {
                                return Err(de::Error::duplicate_field("zfar"));
                            }
                            zfar = Some(map.next_value()?);
                        }
                    }
                }
                let left = left.ok_or_else(|| de::Error::missing_field("left"))?;
                let right = right.ok_or_else(|| de::Error::missing_field("right"))?;
                let bottom = bottom.ok_or_else(|| de::Error::missing_field("bottom"))?;
                let top = top.ok_or_else(|| de::Error::missing_field("top"))?;
                let znear = znear.ok_or_else(|| de::Error::missing_field("znear"))?;
                let zfar = zfar.ok_or_else(|| de::Error::missing_field("zfar"))?;

                Ok(Orthographic3::new(left, right, bottom, top, znear, zfar))
            }
        }

        const FIELDS: &'static [&'static str] =
            &["left", "right", "bottom", "top", "znear", "zfar"];
        deserializer.deserialize_struct("Orthographic", FIELDS, OrthographicVisitor)
    }

    pub fn serialize<S>(proj: &Orthographic3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct OrthographicValues {
            left: f32,
            right: f32,
            bottom: f32,
            top: f32,
            znear: f32,
            zfar: f32,
        }

        Serialize::serialize(
            &OrthographicValues {
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
    use std::fmt;

    use serde::{
        de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
        ser::Serializer,
        Deserialize, Serialize,
    };

    use amethyst_core::nalgebra::Perspective3;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Perspective3<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Aspect,
            Fovy,
            Znear,
            Zfar,
        };

        struct PerspectiveVisitor;

        impl<'de> Visitor<'de> for PerspectiveVisitor {
            type Value = Perspective3<f32>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Perspective")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let aspect = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let fovy = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let znear = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let zfar = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                Ok(Perspective3::new(aspect, fovy, znear, zfar))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut aspect = None;
                let mut fovy = None;
                let mut znear = None;
                let mut zfar = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Aspect => {
                            if aspect.is_some() {
                                return Err(de::Error::duplicate_field("aspect"));
                            }
                            aspect = Some(map.next_value()?);
                        }
                        Field::Fovy => {
                            if fovy.is_some() {
                                return Err(de::Error::duplicate_field("fovy"));
                            }
                            fovy = Some(map.next_value()?);
                        }
                        Field::Znear => {
                            if znear.is_some() {
                                return Err(de::Error::duplicate_field("znear"));
                            }
                            znear = Some(map.next_value()?);
                        }
                        Field::Zfar => {
                            if zfar.is_some() {
                                return Err(de::Error::duplicate_field("zfar"));
                            }
                            zfar = Some(map.next_value()?);
                        }
                    }
                }
                let aspect = aspect.ok_or_else(|| de::Error::missing_field("aspect"))?;
                let fovy = fovy.ok_or_else(|| de::Error::missing_field("fovy"))?;
                let znear = znear.ok_or_else(|| de::Error::missing_field("znear"))?;
                let zfar = zfar.ok_or_else(|| de::Error::missing_field("zfar"))?;

                Ok(Perspective3::new(aspect, fovy, znear, zfar))
            }
        }

        const FIELDS: &'static [&'static str] = &["aspect", "fovy", "znear", "zfar"];
        deserializer.deserialize_struct("Perspective", FIELDS, PerspectiveVisitor)
    }

    pub fn serialize<S>(proj: &Perspective3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct PerspectiveValues {
            aspect: f32,
            fovy: f32,
            znear: f32,
            zfar: f32,
        }

        Serialize::serialize(
            &PerspectiveValues {
                aspect: proj.aspect(),
                fovy: proj.fovy(),
                znear: proj.znear(),
                zfar: proj.zfar(),
            },
            serializer,
        )
    }
}
