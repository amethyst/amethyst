//! Local transform component.
use std::fmt;
use std::marker::PhantomData;

use crate::ecs::prelude::{Component, DenseVecStorage, FlaggedStorage};
use crate::math::{
    self as na, Isometry3, Matrix4, Quaternion, RealField, Translation3, Unit, UnitQuaternion,
    Vector3,
};
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, Serializer},
};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are preformed in this order: scale, then rotation, then translation.
#[derive(Getters, Setters, MutGetters, Clone, Debug, PartialEq)]
pub struct Transform<N: RealField> {
    /// Translation + rotation value
    #[get = "pub"]
    #[set = "pub"]
    #[get_mut = "pub"]
    isometry: Isometry3<N>,
    /// Scale vector
    #[get = "pub"]
    #[set = "pub"]
    #[get_mut = "pub"]
    scale: Vector3<N>,
    /// The global transformation matrix.
    #[get = "pub"]
    pub(crate) global_matrix: Matrix4<N>,
}

impl<N: RealField> Transform<N> {
    /// Create a new Transform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use amethyst_core::transform::components::Transform;
    /// # use amethyst_core::math::{Isometry3, Translation3, UnitQuaternion, Vector3};
    /// let position = Translation3::new(0.0, 2.0, 4.0);
    /// let rotation = UnitQuaternion::from_euler_angles(0.4, 0.2, 0.0);
    /// let scale = Vector3::new(1.0, 1.0, 1.0);
    ///
    /// let t = Transform::<f32>::new(position, rotation, scale);
    ///
    /// assert_eq!(t.translation().y, 2.0);
    /// ```
    pub fn new(position: Translation3<N>, rotation: UnitQuaternion<N>, scale: Vector3<N>) -> Self {
        Transform {
            isometry: Isometry3::from_parts(position, rotation),
            scale,
            global_matrix: na::one(),
        }
    }

    /// Makes the entity point towards `target`.
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
    /// # use amethyst_core::math::{UnitQuaternion, Quaternion, Vector3};
    /// let mut t = Transform::<f32>::default();
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
    pub fn face_towards(&mut self, target: Vector3<N>, up: Vector3<N>) -> &mut Self {
        self.isometry.rotation =
            UnitQuaternion::face_towards(&(self.isometry.translation.vector - target), &up);
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's `GlobalTransform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> Matrix4<N> {
        self.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    /// Returns a reference to the translation vector.
    #[inline]
    pub fn translation(&self) -> &Vector3<N> {
        &self.isometry.translation.vector
    }

    /// Returns a mutable reference to the translation vector.
    #[inline]
    pub fn translation_mut(&mut self) -> &mut Vector3<N> {
        &mut self.isometry.translation.vector
    }

    /// Returns a reference to the rotation quaternion.
    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<N> {
        &self.isometry.rotation
    }

    /// Returns a mutable reference to the rotation quaternion.
    #[inline]
    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<N> {
        &mut self.isometry.rotation
    }

    /// Move relatively to its current position, but the parent's (or
    /// global, if no parent exists) orientation.
    ///
    /// For example, if the object is rotated 45 degrees about its Y axis,
    /// then you *prepend* a translation along the Z axis, it will still
    /// move along the parent's Z axis rather than its local Z axis (which
    /// is rotated 45 degrees).
    #[inline]
    pub fn prepend_translation(&mut self, translation: Vector3<N>) -> &mut Self {
        self.isometry.translation.vector += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// For example, if the object is rotated 45 degrees about its Y axis,
    /// then you append a translation along the Z axis, that Z axis is now
    /// rotated 45 degrees, and so the appended translation will go along that
    /// rotated Z axis.
    ///
    /// Equivalent to rotating the translation by the transform's current
    /// rotation before applying.
    #[inline]
    pub fn append_translation(&mut self, translation: Vector3<N>) -> &mut Self {
        self.isometry.translation.vector += self.isometry.rotation * translation;
        self
    }

    /// Move a distance along an axis relative to the parent's orientation
    /// (or the global orientation if no parent exists).
    ///
    /// For example, if the object is rotated 45 degrees about its Y axis,
    /// then you *prepend* a translation along the Z axis, it will still
    /// move along the parent's Z axis rather than its local Z axis (which
    /// is rotated 45 degrees).
    #[inline]
    pub fn prepend_translation_along(
        &mut self,
        direction: Unit<Vector3<N>>,
        distance: N,
    ) -> &mut Self {
        self.isometry.translation.vector += direction.as_ref() * distance;
        self
    }

    /// Move a distance along an axis relative to the local orientation.
    #[inline]
    pub fn append_translation_along(
        &mut self,
        direction: Unit<Vector3<N>>,
        distance: N,
    ) -> &mut Self {
        self.isometry.translation.vector += self.isometry.rotation * direction.as_ref() * distance;
        self
    }

    /// Move forward relative to current position and orientation.
    #[inline]
    pub fn move_forward(&mut self, amount: N) -> &mut Self {
        // sign is reversed because z comes towards us
        self.append_translation(Vector3::new(N::zero(), N::zero(), -amount))
    }

    /// Move backward relative to current position and orientation.
    #[inline]
    pub fn move_backward(&mut self, amount: N) -> &mut Self {
        self.append_translation(Vector3::new(N::zero(), N::zero(), amount))
    }

    /// Move right relative to current position and orientation.
    #[inline]
    pub fn move_right(&mut self, amount: N) -> &mut Self {
        self.append_translation(Vector3::new(amount, N::zero(), N::zero()))
    }

    /// Move left relative to current position and orientation.
    #[inline]
    pub fn move_left(&mut self, amount: N) -> &mut Self {
        self.append_translation(Vector3::new(-amount, N::zero(), N::zero()))
    }

    /// Move up relative to current position and orientation.
    #[inline]
    pub fn move_up(&mut self, amount: N) -> &mut Self {
        self.append_translation(Vector3::new(N::zero(), amount, N::zero()))
    }

    /// Move down relative to current position and orientation.
    #[inline]
    pub fn move_down(&mut self, amount: N) -> &mut Self {
        self.append_translation(Vector3::new(N::zero(), -amount, N::zero()))
    }

    /// Adds the specified amount to the translation vector's x component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// x axis.
    #[inline]
    pub fn prepend_translation_x(&mut self, amount: N) -> &mut Self {
        self.isometry.translation.vector.x += amount;
        self
    }

    /// Adds the specified amount to the translation vector's y component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// y axis.
    #[inline]
    pub fn prepend_translation_y(&mut self, amount: N) -> &mut Self {
        self.isometry.translation.vector.y += amount;
        self
    }

    /// Adds the specified amount to the translation vector's z component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// z axis.
    #[inline]
    pub fn prepend_translation_z(&mut self, amount: N) -> &mut Self {
        self.isometry.translation.vector.z += amount;
        self
    }

    /// Sets the translation vector's x component to the specified value.
    #[inline]
    pub fn set_translation_x(&mut self, value: N) -> &mut Self {
        self.isometry.translation.vector.x = value;
        self
    }

    /// Sets the translation vector's y component to the specified value.
    #[inline]
    pub fn set_translation_y(&mut self, value: N) -> &mut Self {
        self.isometry.translation.vector.y = value;
        self
    }

    /// Sets the translation vector's z component to the specified value.
    #[inline]
    pub fn set_translation_z(&mut self, value: N) -> &mut Self {
        self.isometry.translation.vector.z = value;
        self
    }

    /// Premultiply a rotation about the x axis, i.e. perform a rotation about
    /// the parent's x axis (or the global x axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_x_axis(&mut self, delta_angle: N) -> &mut Self {
        self.prepend_rotation(Vector3::x_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the x axis, i.e. perform a rotation about
    /// the *local* x-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_x_axis(&mut self, delta_angle: N) -> &mut Self {
        self.append_rotation(Vector3::x_axis(), delta_angle)
    }

    /// Set the rotation about the parent's x axis (or the global x axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_x_axis(&mut self, angle: N) -> &mut Self {
        self.set_rotation_euler(angle, N::zero(), N::zero())
    }

    /// Premultiply a rotation about the y axis, i.e. perform a rotation about
    /// the parent's y axis (or the global y axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_y_axis(&mut self, delta_angle: N) -> &mut Self {
        self.prepend_rotation(Vector3::y_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the y axis, i.e. perform a rotation about
    /// the *local* y-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_y_axis(&mut self, delta_angle: N) -> &mut Self {
        self.append_rotation(Vector3::y_axis(), delta_angle)
    }

    /// Set the rotation about the parent's y axis (or the global y axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_y_axis(&mut self, angle: N) -> &mut Self {
        self.set_rotation_euler(N::zero(), angle, N::zero())
    }

    /// Premultiply a rotation about the z axis, i.e. perform a rotation about
    /// the parent's z axis (or the global z axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_z_axis(&mut self, delta_angle: N) -> &mut Self {
        self.prepend_rotation(-Vector3::z_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the z axis, i.e. perform a rotation about
    /// the *local* z-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_z_axis(&mut self, delta_angle: N) -> &mut Self {
        self.append_rotation(-Vector3::z_axis(), delta_angle)
    }

    /// Set the rotation about the parent's z axis (or the global z axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_z_axis(&mut self, angle: N) -> &mut Self {
        self.set_rotation_euler(N::zero(), N::zero(), angle)
    }

    /// Perform a rotation about the axis perpendicular to X and Y,
    /// i.e. the most common way to rotate an object in a 2d game.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn rotate_2d(&mut self, delta_angle: N) -> &mut Self {
        self.prepend_rotation_z_axis(delta_angle)
    }

    /// Set the rotation about the axis perpendicular to X and Y,
    /// i.e. the most common way to rotate an object in a 2d game.
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_2d(&mut self, angle: N) -> &mut Self {
        self.set_rotation_euler(N::zero(), N::zero(), angle)
    }

    /// Premultiply a rotation, i.e. rotate relatively to the parent's orientation
    /// (or the global orientation if no parent exists), about a specified axis.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation(&mut self, axis: Unit<Vector3<N>>, angle: N) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.isometry.rotation = q * self.isometry.rotation;
        self
    }

    /// Postmultiply a rotation, i.e. rotate relatively to the local orientation (the
    /// currently applied rotations), about a specified axis.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation(&mut self, axis: Unit<Vector3<N>>, angle: N) -> &mut Self {
        self.isometry.rotation *= UnitQuaternion::from_axis_angle(&axis, angle);
        self
    }

    /// Set the position.
    pub fn set_translation(&mut self, position: Vector3<N>) -> &mut Self {
        self.isometry.translation.vector = position;
        self
    }

    /// Adds the specified amounts to the translation vector.
    pub fn append_translation_xyz(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.append_translation(Vector3::new(x, y, z));
        self
    }

    /// Sets the specified values of the translation vector.
    pub fn set_translation_xyz(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.set_translation(Vector3::new(x, y, z))
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: UnitQuaternion<N>) -> &mut Self {
        self.isometry.rotation = rotation;
        self
    }

    /// Set the rotation using x, y, z Euler axes.
    ///
    /// All angles are specified in radians. Euler order is x → y → z.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis.
    ///  - y - The angle to apply around the y axis.
    ///  - z - The angle to apply around the z axis.
    ///
    /// # Note on Euler angle semantics and `nalgebra`
    ///
    /// `nalgebra` has a few methods related to Euler angles, and they use
    /// roll, pitch, and yaw as arguments instead of x, y, and z axes specifically.
    /// Yaw has the semantic meaning of rotation about the "up" axis, roll about the
    /// "forward axis", and pitch about the "right" axis respectively. However, `nalgebra`
    /// assumes a +Z = up coordinate system for its roll, pitch, and yaw semantics, while
    /// Amethyst uses a +Y = up coordinate system. Therefore, the `nalgebra` Euler angle
    /// methods are slightly confusing to use in concert with Amethyst, and so we've
    /// provided our own with semantics that match the rest of Amethyst. If you do end up
    /// using `nalgebra`'s `euler_angles` or `from_euler_angles` methods, be aware that
    /// 'roll' in that context will mean rotation about the x axis, 'pitch' will mean
    /// rotation about the y axis, and 'yaw' will mean rotation about the z axis.
    ///
    /// ```
    /// # use amethyst_core::transform::components::Transform;
    /// let mut transform = Transform::default();
    ///
    /// transform.set_rotation_euler(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(transform.rotation().euler_angles().0, 1.0);
    /// ```
    pub fn set_rotation_euler(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.isometry.rotation = UnitQuaternion::from_euler_angles(x, y, z);
        self
    }

    /// Get the Euler angles of the current rotation. Returns
    /// in a tuple of the form (x, y, z), where `x`, `y`, and `z`
    /// are the current rotation about that axis in radians.
    ///
    /// # Note on Euler angle semantics and `nalgebra`
    ///
    /// `nalgebra` has a few methods related to Euler angles, and they use
    /// roll, pitch, and yaw as arguments instead of x, y, and z axes specifically.
    /// Yaw has the semantic meaning of rotation about the "up" axis, roll about the
    /// "forward axis", and pitch about the "right" axis respectively. However, `nalgebra`
    /// assumes a +Z = up coordinate system for its roll, pitch, and yaw semantics, while
    /// Amethyst uses a +Y = up coordinate system. Therefore, the `nalgebra` Euler angle
    /// methods are slightly confusing to use in concert with Amethyst, and so we've
    /// provided our own with semantics that match the rest of Amethyst. If you do end up
    /// using `nalgebra`'s `euler_angles` or `from_euler_angles` methods, be aware that
    /// 'roll' in that context will mean rotation about the x axis, 'pitch' will mean
    /// rotation about the y axis, and 'yaw' will mean rotation about the z axis.
    pub fn euler_angles(&self) -> (N, N, N) {
        self.isometry.rotation.euler_angles()
    }

    /// Concatenates another transform onto `self`.
    ///
    /// Concatenating is roughly equivalent to doing matrix multiplication except for the fact that
    /// it's done on `Transform` which is decomposed.
    pub fn concat(&mut self, other: &Self) -> &mut Self {
        // The order of these is somewhat important as the translation relies on the rotation and
        // scaling not having been modified already.
        self.isometry.translation.vector +=
            self.isometry.rotation * other.isometry.translation.vector.component_mul(&self.scale);
        self.scale.component_mul_assign(&other.scale);
        self.isometry.rotation *= other.isometry.rotation;
        self
    }

    /// Verifies that the global `Matrix4` doesn't contain any NaN values.
    pub fn is_finite(&self) -> bool {
        self.global_matrix
            .as_slice()
            .iter()
            .all(|f| N::is_finite(f))
    }

    /// Calculates the inverse of this transform, which we need to render.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn view_matrix(&self) -> Matrix4<N> {
        // TODO: check if this actually is faster
        let inv_scale = Vector3::new(
            N::one() / self.scale.x,
            N::one() / self.scale.y,
            N::one() / self.scale.z,
        );
        self.isometry
            .inverse()
            .to_homogeneous()
            .append_nonuniform_scaling(&inv_scale)
    }
}

impl<N: RealField> Default for Transform<N> {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            isometry: Isometry3::identity(),
            scale: Vector3::from_element(N::one()),
            global_matrix: na::one(),
        }
    }
}

impl<N: RealField> Component for Transform<N> {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Creates a Transform using the `Vector3` as the translation vector.
///
/// ```
/// # use amethyst_core::transform::components::Transform;
/// # use amethyst_core::math::Vector3;
/// let transform = Transform::from(Vector3::new(100.0, 200.0, 300.0));
///
/// assert_eq!(transform.translation().x, 100.0);
/// ```
impl<N: RealField> From<Vector3<N>> for Transform<N> {
    fn from(translation: Vector3<N>) -> Self {
        Transform {
            isometry: Isometry3::new(translation, na::zero()),
            ..Default::default()
        }
    }
}

impl<'de, N: RealField + Deserialize<'de>> Deserialize<'de> for Transform<N> {
    fn deserialize<D>(deserializer: D) -> Result<Transform<N>, D::Error>
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

        struct TransformVisitor<N: RealField> {
            _phantom: PhantomData<N>,
        }

        impl<N: RealField> Default for TransformVisitor<N> {
            fn default() -> Self {
                TransformVisitor {
                    _phantom: PhantomData,
                }
            }
        }

        impl<'de, N: RealField + Deserialize<'de>> Visitor<'de> for TransformVisitor<N> {
            type Value = Transform<N>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Transform")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let translation: [N; 3] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let rotation: [N; 4] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let scale: [N; 3] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let isometry = Isometry3::from_parts(
                    Translation3::new(translation[0], translation[1], translation[2]),
                    Unit::new_normalize(Quaternion::new(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    )),
                );
                let scale = scale.into();

                Ok(Transform {
                    isometry,
                    scale,
                    ..Default::default()
                })
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
                let translation: [N; 3] = translation.unwrap_or([N::zero(); 3]);
                let rotation: [N; 4] =
                    rotation.unwrap_or([N::one(), N::zero(), N::zero(), N::zero()]);
                let scale: [N; 3] = scale.unwrap_or([N::one(); 3]);

                let isometry = Isometry3::from_parts(
                    Translation3::new(translation[0], translation[1], translation[2]),
                    Unit::new_normalize(Quaternion::new(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    )),
                );
                let scale = scale.into();

                Ok(Transform {
                    isometry,
                    scale,
                    ..Default::default()
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["translation", "rotation", "scale"];
        deserializer.deserialize_struct("Transform", FIELDS, TransformVisitor::<N>::default())
    }
}

impl<N: RealField + Serialize> Serialize for Transform<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct TransformValues<N: RealField> {
            translation: [N; 3],
            rotation: [N; 4],
            scale: [N; 3],
        }

        Serialize::serialize(
            &TransformValues {
                translation: self.isometry.translation.vector.into(),
                rotation: self.isometry.rotation.as_ref().coords.into(),
                scale: self.scale.into(),
            },
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        approx::*,
        math::{UnitQuaternion, Vector3},
        Transform,
    };

    /// Sanity test for concat operation
    #[test]
    fn test_mul() {
        // For the condition to hold both scales must be uniform
        let mut first = Transform::default();
        first.set_translation_xyz(20., 10., -3.);
        first.set_scale(Vector3::new(2., 2., 2.));
        first.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(-1., 1., 2.), &Vector3::new(1., 0., 0.))
                .unwrap(),
        );

        let mut second = Transform::default();
        second.set_translation_xyz(2., 1., -3.);
        second.set_scale(Vector3::new(1., 1., 1.));
        second.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(7., -1., 3.), &Vector3::new(2., 1., 1.))
                .unwrap(),
        );

        // check Mat(first * second) == Mat(first) * Mat(second)
        assert_relative_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
            max_relative = 0.0000000000001,
        );
        assert_relative_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
            max_relative = 0.0000000000001,
        );
    }

    #[test]
    fn test_view_matrix() {
        let mut transform = Transform::default();
        transform.set_translation_xyz(5.0, 70.1, 43.7);
        transform.set_scale(Vector3::new(1.0, 5.0, 8.9));
        transform.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(-1., 1., 2.), &Vector3::new(1., 0., 0.))
                .unwrap(),
        );

        assert_ulps_eq!(
            transform.matrix().try_inverse().unwrap(),
            transform.view_matrix(),
        );
    }

    #[test]
    fn is_finite() {
        let mut transform = Transform::default();
        assert!(transform.is_finite());

        transform.global_matrix.fill_row(2, std::f32::NAN);
        assert!(!transform.is_finite());
    }
}
