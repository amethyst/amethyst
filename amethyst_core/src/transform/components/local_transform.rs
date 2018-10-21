//! Local transform component.
use std::fmt;

use nalgebra::{
    self as na, Isometry3, Matrix4, Quaternion, Translation3, Unit, UnitQuaternion, Vector3,
};
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, Serializer},
};
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

use orientation::Orientation;

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are preformed in this order: scale, then rotation, then translation.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Translation + rotation value
    pub iso: Isometry3<f32>,
    /// Scale vector
    pub scale: Vector3<f32>,
}

impl Transform {
    /// Makes the entity point towards `position`.
    ///
    /// `up` says which direction the entity should be 'rolled' to once it is pointing at
    /// `position`. If `up` is parallel to the direction the entity is looking, the result will be
    /// garbage.
    ///
    /// This function only works with respect to the coordinate system of its parent, so when used
    /// with an object that's not a sibling it will not do what you expect.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use amethyst_core::transform::components::Transform;
    /// # use amethyst_core::cgmath::{Quaternion, One, Vector3, Point3, Matrix3};
    /// let mut t = Transform::default();
    /// // No rotation by default
    /// assert_eq!(t.rotation, Quaternion::one());
    /// // look up with up pointing backwards
    /// t.look_at(Point3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
    /// // our rotation should match the angle from straight ahead to straight up
    /// let rotation = Quaternion::from_arc(
    ///     Vector3::new(0.0, 0.0, -1.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     None);
    /// assert_eq!(t.rotation, rotation);
    /// ```
    // FIXME doctest
    // TODO: fix example
    #[inline]
    pub fn look_at(&mut self, target: Vector3<f32>, up: Vector3<f32>) -> &mut Self {
        self.iso.rotation =
            UnitQuaternion::look_at_rh(&(self.iso.translation.vector - target), &up);
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's `GlobalTransform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        // This is a hot function, so manually implement the matrix-multiply to avoid a load of
        // unnecessary +0s.
        // Note: Not benchmarked

        // let quat = self.rotation.to_rotation_matrix();
        // let s = quat.matrix().as_slice();

        // let x: Vector4<f32> = Vector4::new(s[0], s[1], s[2], 0.0) * self.scale.x;
        // let y: Vector4<f32> = Vector4::new(s[3], s[4], s[5], 0.0) * self.scale.x;
        // let z: Vector4<f32> = Vector4::new(s[6], s[7], s[8], 0.0) * self.scale.x;
        // let w: Vector4<f32> = self.translation.insert_row(3, 0.0);

        // Matrix4::new(
        //     x.x, x.y, x.z, x.w, // Column 1
        //     y.x, y.y, y.z, y.w, // Column 2
        //     z.x, z.y, z.z, z.w, // Column 3
        //     w.x, w.y, w.z, w.w, // Column 4
        // )

        Matrix4::new_nonuniform_scaling(&self.scale) * self.iso.to_homogeneous()
    }

    #[inline]
    pub fn translation(&self) -> &Vector3<f32> {
        &self.iso.translation.vector
    }

    #[inline]
    pub fn translation_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.iso.translation.vector
    }

    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<f32> {
        &self.iso.rotation
    }

    #[inline]
    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<f32> {
        &mut self.iso.rotation
    }

    /// Convert this transform's rotation into an Orientation, guaranteed to be 3 unit orthogonal
    /// vectors
    pub fn orientation(&self) -> Orientation {
        Orientation::from(*self.iso.rotation.to_rotation_matrix().matrix())
    }

    /// Move relatively to its current position.
    #[inline]
    pub fn move_global(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.iso.translation.vector += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// Equivalent to rotating the translation before applying.
    #[inline]
    pub fn move_local(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * translation;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_global(&mut self, direction: Unit<Vector3<f32>>, distance: f32) -> &mut Self {
        self.iso.translation.vector += direction.as_ref() * (distance / direction.magnitude());
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Unit<Vector3<f32>>, distance: f32) -> &mut Self {
        self.iso.translation.vector +=
            self.iso.rotation * direction.as_ref() * (distance / direction.magnitude());
        self
    }

    /// Move forward relative to current position and orientation.
    #[inline]
    pub fn move_forward(&mut self, amount: f32) -> &mut Self {
        // sign is reversed because z comes towards us
        self.move_local(Vector3::new(0.0, 0.0, -amount))
    }

    /// Move backward relative to current position and orientation.
    #[inline]
    pub fn move_backward(&mut self, amount: f32) -> &mut Self {
        self.move_local(Vector3::new(0.0, 0.0, amount))
    }

    /// Move right relative to current position and orientation.
    #[inline]
    pub fn move_right(&mut self, amount: f32) -> &mut Self {
        self.move_local(Vector3::new(amount, 0.0, 0.0))
    }

    /// Move left relative to current position and orientation.
    #[inline]
    pub fn move_left(&mut self, amount: f32) -> &mut Self {
        self.move_local(Vector3::new(-amount, 0.0, 0.0))
    }

    /// Move up relative to current position and orientation.
    #[inline]
    pub fn move_up(&mut self, amount: f32) -> &mut Self {
        self.move_local(Vector3::new(0.0, amount, 0.0))
    }

    /// Move down relative to current position and orientation.
    #[inline]
    pub fn move_down(&mut self, amount: f32) -> &mut Self {
        self.move_local(Vector3::new(0.0, -amount, 0.0))
    }

    /// Pitch relatively to the world.
    #[inline]
    pub fn pitch_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(Vector3::x_axis(), angle)
    }

    /// Pitch relatively to its own rotation.
    #[inline]
    pub fn pitch_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(Vector3::x_axis(), angle)
    }

    /// Yaw relatively to the world.
    #[inline]
    pub fn yaw_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(Vector3::y_axis(), angle)
    }

    /// Yaw relatively to its own rotation.
    #[inline]
    pub fn yaw_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(Vector3::y_axis(), angle)
    }

    /// Roll relatively to the world.
    #[inline]
    pub fn roll_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(-Vector3::z_axis(), angle)
    }

    /// Roll relatively to its own rotation.
    #[inline]
    pub fn roll_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(-Vector3::z_axis(), angle)
    }

    /// Rotate relatively to the world
    #[inline]
    pub fn rotate_global(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.iso.rotation = q * self.iso.rotation;
        self
    }

    /// Rotate relatively to the current orientation
    #[inline]
    pub fn rotate_local(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.iso.rotation = self.iso.rotation * q;
        self
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.iso.translation.vector = position;
        self
    }

    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) -> &mut Self {
        self.iso.rotation = rotation;
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis. Also known as the pitch.
    ///  - y - The angle to apply around the y axis. Also known as the yaw.
    ///  - z - The angle to apply around the z axis. Also known as the roll.
    pub fn set_rotation_euler(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.iso.rotation = UnitQuaternion::from_euler_angles(z, x, y);
        self
    }

    pub fn concat(&mut self, other: &Self) -> &mut Self {
        self.scale.component_mul_assign(&other.scale);
        self.iso.rotation *= other.iso.rotation;
        self.iso.translation.vector +=
            self.iso.rotation * other.iso.translation.vector.component_mul(&self.scale);
        self
    }

    /// Calculates the inverse of this transform, which we need to render.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        // todo
        self.matrix().try_inverse().unwrap()
    }
}

impl Default for Transform {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            iso: Isometry3::from_parts(Translation3::from_vector(na::zero()), na::one()),
            scale: Vector3::from_element(1.0),
        }
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Creates a Transform using the `Vector3` as the translation vector.
impl From<Vector3<f32>> for Transform {
    fn from(translation: Vector3<f32>) -> Self {
        Transform {
            iso: Isometry3::from_parts(Translation3::from_vector(translation), na::one()),
            ..Default::default()
        }
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Transform, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Translation,
            Rotation,
            Scale,
        };

        struct TransformVisitor;

        impl<'de> Visitor<'de> for TransformVisitor {
            type Value = Transform;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Transform")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let translation: [f32; 3] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let rotation: [f32; 4] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let scale: [f32; 3] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let iso = Isometry3::from_parts(
                    Translation3::from_vector(translation.into()),
                    Unit::new_normalize(Quaternion::new(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    )),
                );
                let scale = scale.into();

                Ok(Transform { iso, scale })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut translation = None;
                let mut rotation = None;
                let mut scale = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Translation => {
                            if translation.is_some() {
                                return Err(de::Error::duplicate_field("translation"));
                            }
                            translation = Some(map.next_value()?);
                        }
                        Field::Rotation => {
                            if rotation.is_some() {
                                return Err(de::Error::duplicate_field("rotation"));
                            }
                            rotation = Some(map.next_value()?);
                        }
                        Field::Scale => {
                            if scale.is_some() {
                                return Err(de::Error::duplicate_field("scale"));
                            }
                            scale = Some(map.next_value()?);
                        }
                    }
                }
                let translation: [f32; 3] = translation.unwrap_or([0.0; 3]);
                let rotation: [f32; 4] = rotation.unwrap_or([1.0, 0.0, 0.0, 0.0]);
                let scale: [f32; 3] = scale.unwrap_or([1.0; 3]);

                let iso = Isometry3::from_parts(
                    Translation3::from_vector(translation.into()),
                    Unit::new_normalize(Quaternion::new(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    )),
                );
                let scale = scale.into();

                eprintln!("iso, scale = {:?} {:?}", iso, scale);

                Ok(Transform { iso, scale })
            }
        }

        const FIELDS: &'static [&'static str] = &["translation", "rotation", "scale"];
        deserializer.deserialize_struct("Transform", FIELDS, TransformVisitor)
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct TransformValues {
            translation: [f32; 3],
            rotation: [f32; 4],
            scale: [f32; 3],
        }

        Serialize::serialize(
            &TransformValues {
                translation: self.iso.translation.vector.into(),
                rotation: self.iso.rotation.as_ref().coords.into(),
                scale: self.scale.into(),
            },
            serializer,
        )
    }
}
