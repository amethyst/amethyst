//! Local transform component.
use std::fmt;
use std::marker::PhantomData;

use nalgebra::{
    self as na, Isometry3, Matrix4, Quaternion, RealField, Translation3, Unit, UnitQuaternion, Vector3,
};
use math::prelude::{Component, DenseVecStorage, FlaggedStorage};
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
    iso: Isometry3<N>,
    /// Scale vector
    #[get = "pub"]
    #[set = "pub"]
    #[get_mut = "pub"]
    scale: Vector3<N>,
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
    /// # use amethyst_core::nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};
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
            iso: Isometry3::from_parts(position, rotation),
            scale,
            global_matrix: na::one(),
        }
    }

    // TODO!!!!!!!!!!!!!!-----------------------------------------
    /*#[inline]
    pub fn face_towards_2d(&mut self, target: Vector2<N>) -> &mut Self {
        let wanted_direction = Unit::new_normalize(target - self.iso.translation.vector); // NEEDS TO BE UNIT
        let angle = wanted_direction.y.atan2(wanted_direction.x);
        self.iso.rotation = UnitComplex::new(angle);
        self
    }*/

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
    /// // now if we move forwards by N::one(), we'll end up at the point we are facing
    /// // (modulo some floating point error)
    /// t.move_forward(1.0);
    /// assert!((*t.translation() - Vector3::new(0.0, 1.0, 0.0)).magnitude() <= 0.0001);
    /// ```
    #[inline]
    pub fn face_towards(&mut self, target: Vector3<N>, up: Vector3<N>) -> &mut Self {
        self.iso.rotation =
            UnitQuaternion::face_towards(&(self.iso.translation.vector - target), &up);
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the `global_matrix` field from the parent's `Transform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> Matrix4<N> {
        self.iso
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    /// Returns a reference to the translation vector.
    #[inline]
    pub fn translation(&self) -> &Vector3<N> {
        &self.iso.translation.vector
    }

    /// Returns a mutable reference to the translation vector.
    #[inline]
    pub fn translation_mut(&mut self) -> &mut Vector3<N> {
        &mut self.iso.translation.vector
    }

    /// Returns a reference to the rotation quaternion.
    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<N> {
        &self.iso.rotation
    }

    /// Returns a mutable reference to the rotation quaternion.
    #[inline]
    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<N> {
        &mut self.iso.rotation
    }

    /// Returns a reference to the isometry of the transform (translation and rotation combined).
    #[inline]
    pub fn isometry(&self) -> &Isometry3<N> {
        &self.iso
    }

    /// Returns a mutable reference to the isometry of the transform (translation and rotation
    /// combined).
    #[inline]
    pub fn isometry_mut(&mut self) -> &mut Isometry3<N> {
        &mut self.iso
    }

    /// Move relatively to its current position.
    #[inline]
    pub fn move_global(&mut self, translation: Vector3<N>) -> &mut Self {
        self.iso.translation.vector += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// Equivalent to rotating the translation before applying.
    #[inline]
    pub fn move_local(&mut self, translation: Vector3<N>) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * translation;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_global(&mut self, direction: Unit<Vector3<N>>, distance: N) -> &mut Self {
        self.iso.translation.vector += direction.as_ref() * distance;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Unit<Vector3<N>>, distance: N) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * direction.as_ref() * distance;
        self
    }

    /// Move forward relative to current position and orientation.
    #[inline]
    pub fn move_forward(&mut self, amount: N) -> &mut Self {
        // sign is reversed because z comes towards us
        self.move_local(Vector3::new(N::zero(), N::zero(), -amount))
    }

    /// Move backward relative to current position and orientation.
    #[inline]
    pub fn move_backward(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector3::new(N::zero(), N::zero(), amount))
    }

    /// Move right relative to current position and orientation.
    #[inline]
    pub fn move_right(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector3::new(amount, N::zero(), N::zero()))
    }

    /// Move left relative to current position and orientation.
    #[inline]
    pub fn move_left(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector3::new(-amount, N::zero(), N::zero()))
    }

    /// Move up relative to current position and orientation.
    #[inline]
    pub fn move_up(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector3::new(N::zero(), amount, N::zero()))
    }

    /// Move down relative to current position and orientation.
    #[inline]
    pub fn move_down(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector3::new(N::zero(), -amount, N::zero()))
    }

    /// Adds the specified amount to the translation vector's x component.
    #[inline]
    pub fn translate_x(&mut self, amount: N) -> &mut Self {
        self.iso.translation.vector.x += amount;
        self
    }

    /// Adds the specified amount to the translation vector's y component.
    #[inline]
    pub fn translate_y(&mut self, amount: N) -> &mut Self {
        self.iso.translation.vector.y += amount;
        self
    }

    /// Adds the specified amount to the translation vector's z component.
    #[inline]
    pub fn translate_z(&mut self, amount: N) -> &mut Self {
        self.iso.translation.vector.z += amount;
        self
    }

    /// Sets the translation vector's x component to the specified value.
    #[inline]
    pub fn set_x(&mut self, value: N) -> &mut Self {
        self.iso.translation.vector.x = value;
        self
    }

    /// Sets the translation vector's y component to the specified value.
    #[inline]
    pub fn set_y(&mut self, value: N) -> &mut Self {
        self.iso.translation.vector.y = value;
        self
    }

    /// Sets the translation vector's z component to the specified value.
    #[inline]
    pub fn set_z(&mut self, value: N) -> &mut Self {
        self.iso.translation.vector.z = value;
        self
    }

    /// Pitch relatively to the world. `angle` is specified in radians.
    /// Rotation relative to the Vector3::x_axis() axis.
    #[inline]
    pub fn pitch_global(&mut self, angle: N) -> &mut Self {
        self.rotate_global(Vector3::x_axis(), angle)
    }

    /// Pitch relatively to its own rotation. `angle` is specified in radians.
    /// Rotation relative to the Vector3::x_axis() axis.
    #[inline]
    pub fn pitch_local(&mut self, angle: N) -> &mut Self {
        self.rotate_local(Vector3::x_axis(), angle)
    }

    /// Yaw relatively to the world. `angle` is specified in radians.
    /// Rotation relative to the Vector3::y_axis() axis.
    #[inline]
    pub fn yaw_global(&mut self, angle: N) -> &mut Self {
        self.rotate_global(Vector3::y_axis(), angle)
    }

    /// Yaw relatively to its own rotation. `angle` is specified in radians.
    /// Rotation relative to the Vector3::y_axis() axis.
    #[inline]
    pub fn yaw_local(&mut self, angle: N) -> &mut Self {
        self.rotate_local(Vector3::y_axis(), angle)
    }

    /// Roll relatively to the world. `angle` is specified in radians.
    /// Rotation relative to the Vector3::z_axis() axis.
    #[inline]
    pub fn roll_global(&mut self, angle: N) -> &mut Self {
        self.rotate_global(-Vector3::z_axis(), angle)
    }

    /// Roll relatively to its own rotation. `angle` is specified in radians.
    /// Rotation relative to the Vector3::z_axis() axis.
    #[inline]
    pub fn roll_local(&mut self, angle: N) -> &mut Self {
        self.rotate_local(-Vector3::z_axis(), angle)
    }

    /// Rotate relatively to the world. `angle` is specified in radians.
    #[inline]
    pub fn rotate_global(&mut self, axis: Unit<Vector3<N>>, angle: N) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.iso.rotation = q * self.iso.rotation;
        self
    }

    /// Rotate relatively to the current orientation. `angle` is specified in radians.
    #[inline]
    pub fn rotate_local(&mut self, axis: Unit<Vector3<N>>, angle: N) -> &mut Self {
        self.iso.rotation *= UnitQuaternion::from_axis_angle(&axis, angle);
        self
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Vector3<N>) -> &mut Self {
        self.iso.translation.vector = position;
        self
    }

    /// Adds the specified amounts to the translation vector.
    pub fn translate_xyz(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.translate_x(x);
        self.translate_y(y);
        self.translate_z(z);
        self
    }

    /// Sets the specified values of the translation vector.
    pub fn set_xyz(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.set_position(Vector3::new(x, y, z))
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: UnitQuaternion<N>) -> &mut Self {
        self.iso.rotation = rotation;
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// All angles are specified in radians. Euler order is roll → pitch → yaw.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis. Also known as the roll.
    ///  - y - The angle to apply around the y axis. Also known as the pitch.
    ///  - z - The angle to apply around the z axis. Also known as the yaw.
    /// ```
    /// # use amethyst_core::transform::components::Transform;
    /// let mut transform = Transform::default();
    ///
    /// transform.set_rotation_euler(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(transform.rotation().euler_angles().0, 1.0);
    /// ```
    pub fn set_rotation_euler(&mut self, x: N, y: N, z: N) -> &mut Self {
        self.iso.rotation = UnitQuaternion::from_euler_angles(x, y, z);
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
        self.iso
            .inverse()
            .to_homogeneous()
            .append_nonuniform_scaling(&inv_scale)
    }
}

impl<N: RealField> Default for Transform<N> {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            iso: Isometry3::identity(),
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
/// # use amethyst_core::nalgebra::Vector3;
/// let transform = Transform::from(Vector3::new(100.0, 200.0, 300.0));
///
/// assert_eq!(transform.translation().x, 100.0);
/// ```
impl<N: RealField> From<Vector3<N>> for Transform<N> {
    fn from(translation: Vector3<N>) -> Self {
        Transform {
            iso: Isometry3::new(translation, na::zero()),
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

                Ok(Transform {
                    iso,
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

                Ok(Transform {
                    iso,
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
    use crate::{
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
        first.set_scale(Vector3::new(2., 2., 2.));
        first.set_rotation(
            UnitQuaternion::rotation_between(&Vector3::new(-1., 1., 2.), &Vector3::new(1., 0., 0.))
                .unwrap(),
        );

        let mut second = Transform::default();
        second.set_xyz(2., 1., -3.);
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
        transform.set_xyz(5.0, 70.1, 43.7);
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
