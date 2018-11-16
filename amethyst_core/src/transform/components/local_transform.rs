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

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are preformed in this order: scale, then rotation, then translation.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Translation + rotation value
    iso: Isometry3<f32>,
    /// Scale vector
    scale: Vector3<f32>,
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
    /// ```rust
    /// # use amethyst_core::transform::components::Transform;
    /// # use amethyst_core::nalgebra::{UnitQuaternion, Quaternion, Vector3};
    /// let mut t = Transform::default();
    /// // No rotation by default
    /// assert_eq!(*t.rotation().quaternion(), Quaternion::identity());
    /// // look up with up pointing backwards
    /// t.face_towards(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
    /// // our rotation should match the angle from straight ahead to straight up
    /// let rotation = UnitQuaternion::rotation_between(
    ///     &Vector3::new(0.0, 1.0, 0.0),
    ///     &Vector3::new(0.0, 0.0, 1.0),
    /// ).unwrap();
    /// assert_eq!(*t.rotation(), rotation);
    /// // now if we move forwards by 1.0, we'll end up at the point we are facing
    /// // (modulo some floating point error)
    /// t.move_forward(1.0);
    /// assert!((*t.translation() - Vector3::new(0.0, 1.0, 0.0)).magnitude() <= 0.0001);
    /// ```
    #[inline]
    pub fn face_towards(&mut self, target: Vector3<f32>, up: Vector3<f32>) -> &mut Self {
        self.iso.rotation =
            UnitQuaternion::new_observer_frame(&(self.iso.translation.vector - target), &up);
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's `GlobalTransform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        self.iso
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    /// Returns a reference to the translation vector.
    #[inline]
    pub fn translation(&self) -> &Vector3<f32> {
        &self.iso.translation.vector
    }

    /// Returns a mutable reference to the translation vector.
    #[inline]
    pub fn translation_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.iso.translation.vector
    }

    /// Returns a reference to the rotation quaternion.
    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<f32> {
        &self.iso.rotation
    }

    /// Returns a mutable reference to the rotation quaternion.
    #[inline]
    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<f32> {
        &mut self.iso.rotation
    }

    /// Returns a reference to the scale vector.
    #[inline]
    pub fn scale(&self) -> &Vector3<f32> {
        &self.scale
    }

    /// Returns a mutable reference to the scale vector.
    #[inline]
    pub fn scale_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.scale
    }

    /// Returns a reference to the isometry of the transform (translation and rotation combined).
    #[inline]
    pub fn isometry(&self) -> &Isometry3<f32> {
        &self.iso
    }

    /// Returns a mutable reference to the isometry of the transform (translation and rotation
    /// combined).
    #[inline]
    pub fn isometry_mut(&mut self) -> &mut Isometry3<f32> {
        &mut self.iso
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
        self.iso.translation.vector += direction.as_ref() * distance;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Unit<Vector3<f32>>, distance: f32) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * direction.as_ref() * distance;
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

    /// Adds the specified amount to the translation vector's x component.
    #[inline]
    pub fn translate_x(&mut self, amount: f32) -> &mut Self {
        self.iso.translation.vector.x += amount;
        self
    }

    /// Adds the specified amount to the translation vector's y component.
    #[inline]
    pub fn translate_y(&mut self, amount: f32) -> &mut Self {
        self.iso.translation.vector.y += amount;
        self
    }

    /// Adds the specified amount to the translation vector's z component.
    #[inline]
    pub fn translate_z(&mut self, amount: f32) -> &mut Self {
        self.iso.translation.vector.z += amount;
        self
    }

    /// Sets the translation vector's x component to the specified value.
    #[inline]
    pub fn set_x(&mut self, value: f32) -> &mut Self {
        self.iso.translation.vector.x = value;
        self
    }

    /// Sets the translation vector's y component to the specified value.
    #[inline]
    pub fn set_y(&mut self, value: f32) -> &mut Self {
        self.iso.translation.vector.y = value;
        self
    }

    /// Sets the translation vector's z component to the specified value.
    #[inline]
    pub fn set_z(&mut self, value: f32) -> &mut Self {
        self.iso.translation.vector.z = value;
        self
    }

    /// Pitch relatively to the world. `angle` is specified in radians.
    #[inline]
    pub fn pitch_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(Vector3::x_axis(), angle)
    }

    /// Pitch relatively to its own rotation. `angle` is specified in radians.
    #[inline]
    pub fn pitch_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(Vector3::x_axis(), angle)
    }

    /// Yaw relatively to the world. `angle` is specified in radians.
    #[inline]
    pub fn yaw_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(Vector3::y_axis(), angle)
    }

    /// Yaw relatively to its own rotation. `angle` is specified in radians.
    #[inline]
    pub fn yaw_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(Vector3::y_axis(), angle)
    }

    /// Roll relatively to the world. `angle` is specified in radians.
    #[inline]
    pub fn roll_global(&mut self, angle: f32) -> &mut Self {
        self.rotate_global(-Vector3::z_axis(), angle)
    }

    /// Roll relatively to its own rotation. `angle` is specified in radians.
    #[inline]
    pub fn roll_local(&mut self, angle: f32) -> &mut Self {
        self.rotate_local(-Vector3::z_axis(), angle)
    }

    /// Rotate relatively to the world. `angle` is specified in radians.
    #[inline]
    pub fn rotate_global(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.iso.rotation = q * self.iso.rotation;
        self
    }

    /// Rotate relatively to the current orientation. `angle` is specified in radians.
    #[inline]
    pub fn rotate_local(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        self.iso.rotation *= UnitQuaternion::from_axis_angle(&axis, angle);
        self
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.iso.translation.vector = position;
        self
    }

    /// Adds the specified amounts to the translation vector.
    pub fn translate_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.translate_x(x);
        self.translate_y(y);
        self.translate_z(z);
        self
    }

    /// Sets the specified values of the translation vector.
    pub fn set_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.set_position(Vector3::new(x, y, z))
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) -> &mut Self {
        self.iso.rotation = rotation;
        self
    }

    /// Sets the scale of the transform.
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.scale.x = x;
        self.scale.y = y;
        self.scale.z = z;
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// All angles are specified in radians.
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

    /// Concatenates another transform onto `self`.
    ///
    /// Concatenating is roughly equivalent to doing matrix multiplication except for the fact that
    /// it's done on `Transform` which is decomposed.
    pub fn concat(&mut self, other: &Self) -> &mut Self {
        // The order of these is somewhat important as the translation relies on the rotation and
        // scaling not having been modified already.
        self.iso.translation.vector +=
            self.iso.rotation * other.iso.translation.vector.component_mul(&self.scale);
        self.scale.component_mul_assign(&other.scale);
        self.iso.rotation *= other.iso.rotation;
        self
    }

    /// Calculates the inverse of this transform, which we need to render.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        // TODO: check if this actually is faster
        let inv_scale = Vector3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        self.iso
            .inverse()
            .to_homogeneous()
            .append_nonuniform_scaling(&inv_scale)
    }
}

impl Default for Transform {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            iso: Isometry3::identity(),
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
            iso: Isometry3::new(translation, na::zero()),
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
        #[serde(field_identifier, rename_all = "snake_case")]
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
                    Translation3::new(translation[0], translation[1], translation[2]),
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
                    Translation3::new(translation[0], translation[1], translation[2]),
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

#[cfg(test)]
mod tests {
    use {
        approx::*,
        nalgebra::{UnitQuaternion, Vector3},
        Transform,
    };

    /// Sanity test for concat operation
    #[test]
    fn test_mul() {
        // For the condition to hold both scales must be uniform
        let mut first = Transform::default();
        first.set_xyz(20., 10., -3.);
        first.set_scale(2., 2., 2.);
        first.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(-1., 1., 2.), &Vector3::new(1., 0., 0.))
                .unwrap(),
        );

        let mut second = Transform::default();
        second.set_xyz(2., 1., -3.);
        second.set_scale(1., 1., 1.);
        second.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(7., -1., 3.), &Vector3::new(2., 1., 1.))
                .unwrap(),
        );

        // check Mat(first * second) == Mat(first) * Mat(second)
        assert_ulps_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
        );
        assert_ulps_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
        );
    }

    #[test]
    fn test_view_matrix() {
        let mut transform = Transform::default();
        transform.set_xyz(5.0, 70.1, 43.7);
        transform.set_scale(1.0, 5.0, 8.9);
        transform.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(-1., 1., 2.), &Vector3::new(1., 0., 0.))
                .unwrap(),
        );

        assert_ulps_eq!(
            transform.matrix().try_inverse().unwrap(),
            transform.view_matrix(),
        );
    }
}
