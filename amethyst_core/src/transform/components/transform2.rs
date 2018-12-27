//! Local transform component.
use std::marker::PhantomData;
use std::fmt;

use nalgebra::{
    self as na, Isometry2, Matrix3, Quaternion, Translation2, Unit, UnitQuaternion, Vector2, Real, UnitComplex,
};
use nalgebra::num_complex::Complex;
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
#[derive(Getters, Setters, MutGetters, Clone, Debug, PartialEq, new)]
pub struct Transform2<N: Real> {
    #[get] #[set] #[get_mut]
    iso: Isometry2<N>,
    #[get] #[set] #[get_mut]
    scale: Vector2<N>,
    #[get] #[set] #[get_mut]
    dimensions: Vector2<N>,
    #[get] #[set] #[get_mut]
    layer: i32,
}

impl<N: Real> Transform2<N> {
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
    /// t.face_towards(Vector3::new(N::zero(), N::one(), N::zero()), Vector3::new(N::zero(), N::zero(), N::one()));
    /// // our rotation should match the angle from straight ahead to straight up
    /// let rotation = UnitQuaternion::rotation_between(
    ///     &Vector3::new(N::zero(), N::one(), N::zero()),
    ///     &Vector3::new(N::zero(), N::zero(), N::one()),
    /// ).unwrap();
    /// assert_eq!(*t.rotation(), rotation);
    /// // now if we move forwards by N::one(), we'll end up at the point we are facing
    /// // (modulo some floating point error)
    /// t.move_forward(N::one());
    /// assert!((*t.translation() - Vector3::new(N::zero(), N::one(), N::zero())).magnitude() <= N::zero()001);
    /// ```
    /*#[inline]
    pub fn face_towards(&mut self, target: Vector2<N>) -> &mut Self {
        self.iso.rotation =
            UnitQuaternion::new_observer_frame(&(self.iso.translation.vector - target), &up);
        self
    }
*/
//TODO

    /// Move relatively to its current position.
    #[inline]
    pub fn move_global(&mut self, translation: Vector2<N>) -> &mut Self {
        self.iso.translation.vector += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// Equivalent to rotating the translation before applying.
    #[inline]
    pub fn move_local(&mut self, translation: Vector2<N>) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * translation;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_global(&mut self, direction: Unit<Vector2<N>>, distance: N) -> &mut Self {
        self.iso.translation.vector += direction.as_ref() * distance;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Unit<Vector2<N>>, distance: N) -> &mut Self {
        self.iso.translation.vector += self.iso.rotation * direction.as_ref() * distance;
        self
    }

    /// Move right relative to current position and orientation.
    #[inline]
    pub fn move_right(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector2::new(amount, N::zero()))
    }

    /// Move left relative to current position and orientation.
    #[inline]
    pub fn move_left(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector2::new(-amount, N::zero()))
    }

    /// Move up relative to current position and orientation.
    #[inline]
    pub fn move_up(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector2::new(N::zero(), amount))
    }

    /// Move down relative to current position and orientation.
    #[inline]
    pub fn move_down(&mut self, amount: N) -> &mut Self {
        self.move_local(Vector2::new(N::zero(), -amount))
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

    /// Set the position.
    pub fn set_position(&mut self, position: Vector2<N>) -> &mut Self {
        self.iso.translation.vector = position;
        self
    }

    /// Adds the specified amounts to the translation vector.
    pub fn translate_xy(&mut self, x: N, y: N) -> &mut Self {
        self.translate_x(x);
        self.translate_y(y);
        self
    }

    /// Sets the specified values of the translation vector.
    pub fn set_xy(&mut self, x: N, y: N) -> &mut Self {
        self.set_position(Vector2::new(x, y))
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: UnitComplex<N>) -> &mut Self {
        self.iso.rotation = rotation;
        self
    }

    /// Sets the scale of the transform.
    pub fn set_scale(&mut self, x: N, y: N) -> &mut Self {
        self.scale.x = x;
        self.scale.y = y;
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
    pub fn view_matrix(&self) -> Matrix3<N> {
        // TODO: check if this actually is faster
        let inv_scale = Vector2::new(N::one() / self.scale.x, N::one() / self.scale.y);
        self.iso
            .inverse()
            .to_homogeneous()
            .append_nonuniform_scaling(&inv_scale)
    }
}

impl<N: Real> Default for Transform2<N> {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform2 {
            iso: Isometry2::identity(),
            scale: Vector2::from_element(N::one()),
            dimensions: Vector2::from_element(N::one()),
            layer: 0,
        }
    }
}

impl<N: Real> Component for Transform2<N> {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Creates a Transform using the `Vector3` as the translation vector.
///
/// ```
/// # use amethyst_core::transform::components::Transform;
/// # use amethyst_core::nalgebra::Vector3;
/// let transform = Transform::from(Vector3::new(10N::zero(), 20N::zero(), 30N::zero()));
///
/// assert_eq!(transform.translation().x, 10N::zero());
/// ```
impl<N: Real> From<Vector2<N>> for Transform2<N> {
    fn from(translation: Vector2<N>) -> Self {
        Transform2 {
            iso: Isometry2::new(translation, na::zero()),
            ..Default::default()
        }
    }
}

impl<'de, N: Real + Deserialize<'de>> Deserialize<'de> for Transform2<N> {
    fn deserialize<D>(deserializer: D) -> Result<Transform2<N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Translation,
            Rotation,
            Scale,
            Dimensions,
            Layer,
        };

        struct TransformVisitor<N: Real>{
            _phantom: PhantomData<N>,
        }

        impl<N: Real> Default for TransformVisitor<N> {
            fn default() -> Self {
                TransformVisitor {
                    _phantom: PhantomData,
                }
            }
        }

        impl<'de, N: Real + Deserialize<'de>> Visitor<'de> for TransformVisitor<N> {
            type Value = Transform2<N>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Transform2")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let translation: [N; 2] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let rotation: N = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let scale: [N; 2] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let dimensions: [N; 2] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let layer: i32 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let iso = Isometry2::from_parts(
                    Translation2::new(translation[0], translation[1]),
                    UnitComplex::from_angle(rotation),
                );
                let scale = scale.into();
                let dimensions = dimensions.into();

                Ok(Transform2 { iso, scale, dimensions, layer })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut translation = None;
                let mut rotation = None;
                let mut scale = None;
                let mut dimensions = None;
                let mut layer = None;

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
                        Field::Dimensions => {
                            if dimensions.is_some() {
                                return Err(de::Error::duplicate_field("dimensions"));
                            }
                            dimensions = Some(map.next_value()?);
                        }
                        Field::Layer => {
                            if layer.is_some() {
                                return Err(de::Error::duplicate_field("layer"));
                            }
                            layer = Some(map.next_value()?);
                        }
                    }
                }
                let translation: [N; 2] = translation.unwrap_or([N::zero(); 2]);
                let rotation: N = rotation.unwrap_or(N::zero());
                let scale: [N; 2] = scale.unwrap_or([N::one(); 2]);
                let dimensions: [N; 2] = dimensions.unwrap_or([N::one(); 2]);
                let layer: i32 = layer.unwrap_or(0);

                let iso = Isometry2::from_parts(
                    Translation2::new(translation[0], translation[1]),
                    UnitComplex::from_angle(rotation),
                );
                let scale = scale.into();
                let dimensions = dimensions.into();

                Ok(Transform2 { iso, scale, dimensions, layer })
            }
        }

        const FIELDS: &'static [&'static str] = &["translation", "rotation", "scale", "dimensions", "layer"];
        deserializer.deserialize_struct("Transform2", FIELDS, TransformVisitor::<N>::default())
    }
}

impl<N: Real + Serialize> Serialize for Transform2<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct TransformValues<N: Real> {
            translation: [N; 2],
            rotation: N,
            scale: [N; 2],
            dimensions: [N; 2],
            layer: i32,
        }

        Serialize::serialize(
            &TransformValues {
                translation: self.iso.translation.vector.into(),
                rotation: self.iso.rotation.angle(),
                scale: self.scale.into(),
                dimensions: self.dimensions.into(),
                layer: self.layer,
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
        transform.set_scale(N::one(), 5.0, 8.9);
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
