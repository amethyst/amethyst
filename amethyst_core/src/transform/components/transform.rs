//! Local transform component.
use getset::*;
use legion_prefab::register_component_type;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use simba::scalar::SubsetOf;
use type_uuid::TypeUuid;

use crate::math::{
    self as na, Isometry3, Matrix4, Quaternion, RealField, Translation3, Unit, UnitQuaternion,
    Vector3,
};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are performed in this order: scale, then rotation, then translation.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Getters,
    MutGetters,
    PartialEq,
    Serialize,
    Setters,
    TypeUuid,
    SerdeDiff,
)]
#[uuid = "e20afc7a-6de0-4ea4-95b7-1a6583425208"]
#[serde(from = "TransformValues", into = "TransformValues")]
pub struct Transform {
    /// Translation + rotation value
    #[get = "pub"]
    #[set = "pub"]
    #[get_mut = "pub"]
    #[serde_diff(opaque)]
    isometry: Isometry3<f32>,
    /// Scale vector
    #[get = "pub"]
    #[get_mut = "pub"]
    #[serde_diff(opaque)]
    scale: Vector3<f32>,
    /// The global transformation matrix.
    #[get = "pub"]
    #[serde_diff(opaque)]
    pub(crate) global_matrix: Matrix4<f32>,
    /// The parent transformation matrix.
    #[get = "pub"]
    #[serde_diff(opaque)]
    pub(crate) parent_matrix: Matrix4<f32>,
}

impl Transform {
    /// Create a new Transform.
    ///
    /// # Examples
    ///
    /// ```
    /// # use amethyst::core::transform::Transform;
    /// # use amethyst::core::math::{Isometry3, Translation3, UnitQuaternion, Vector3};
    /// let position = Translation3::new(0.0, 2.0, 4.0);
    /// let rotation = UnitQuaternion::from_euler_angles(0.4, 0.2, 0.0);
    /// let scale = Vector3::new(1.0, 1.0, 1.0);
    ///
    /// let t = Transform::new(position, rotation, scale);
    ///
    /// assert_eq!(t.translation().y, 2.0);
    /// ```
    pub fn new<N: RealField + SubsetOf<f32>>(
        position: Translation3<N>,
        rotation: UnitQuaternion<N>,
        scale: Vector3<N>,
    ) -> Self {
        Transform {
            isometry: Isometry3::from_parts(na::convert(position), na::convert(rotation)),
            scale: na::convert(scale),
            global_matrix: na::one(),
            parent_matrix: na::one(),
        }
    }

    /// Set the scaling factor of this transform.
    pub fn set_scale<N: RealField + SubsetOf<f32>>(&mut self, scale: Vector3<N>) {
        self.scale = na::convert(scale);
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
    /// ```
    /// # use amethyst::core::transform::Transform;
    /// # use amethyst::core::math::{UnitQuaternion, Quaternion, Vector3};
    /// let mut t = Transform::default();
    /// // No rotation by default
    /// assert_eq!(*t.rotation().quaternion(), Quaternion::identity());
    /// // look up with up pointing backwards
    /// t.face_towards(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
    /// // our rotation should match the angle from straight ahead to straight up
    /// let rotation = UnitQuaternion::rotation_between(
    ///     &Vector3::new(0.0, 1.0, 0.0),
    ///     &Vector3::new(0.0, 0.0, 1.0),
    /// )
    /// .unwrap();
    /// assert_eq!(*t.rotation(), rotation);
    /// // now if we move forwards by 1.0, we'll end up at the point we are facing
    /// // (modulo some floating point error)
    /// t.move_forward(1.0);
    /// assert!((*t.translation() - Vector3::new(0.0, 1.0, 0.0)).magnitude() <= 0.0001);
    /// ```
    #[inline]
    pub fn face_towards<N: RealField + SubsetOf<f32>>(
        &mut self,
        target: Vector3<N>,
        up: Vector3<N>,
    ) -> &mut Self {
        self.isometry.rotation = UnitQuaternion::face_towards(
            &(self.isometry.translation.vector - na::convert::<_, Vector3<f32>>(target)),
            &na::convert::<_, Vector3<f32>>(up),
        );
        self
    }

    /// Returns the local object matrix for the transform.
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        self.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    /// Returns a reference to the translation vector.
    #[inline]
    pub fn translation(&self) -> &Vector3<f32> {
        &self.isometry.translation.vector
    }

    /// Returns a mutable reference to the translation vector.
    #[inline]
    pub fn translation_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.isometry.translation.vector
    }

    /// Returns a reference to the rotation quaternion.
    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<f32> {
        &self.isometry.rotation
    }

    /// Returns a mutable reference to the rotation quaternion.
    #[inline]
    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<f32> {
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
    pub fn prepend_translation(&mut self, translation: Vector3<f32>) -> &mut Self {
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
    pub fn append_translation(&mut self, translation: Vector3<f32>) -> &mut Self {
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
        direction: Unit<Vector3<f32>>,
        distance: f32,
    ) -> &mut Self {
        self.isometry.translation.vector += direction.as_ref() * distance;
        self
    }

    /// Move a distance along an axis relative to the local orientation.
    #[inline]
    pub fn append_translation_along(
        &mut self,
        direction: Unit<Vector3<f32>>,
        distance: f32,
    ) -> &mut Self {
        self.isometry.translation.vector += self.isometry.rotation * direction.as_ref() * distance;
        self
    }

    /// Move forward relative to current position and orientation.
    #[inline]
    pub fn move_forward(&mut self, amount: f32) -> &mut Self {
        // sign is reversed because z comes towards us
        self.append_translation(Vector3::new(0.0, 0.0, -amount))
    }

    /// Move backward relative to current position and orientation.
    #[inline]
    pub fn move_backward(&mut self, amount: f32) -> &mut Self {
        self.append_translation(Vector3::new(0.0, 0.0, amount))
    }

    /// Move right relative to current position and orientation.
    #[inline]
    pub fn move_right(&mut self, amount: f32) -> &mut Self {
        self.append_translation(Vector3::new(amount, 0.0, 0.0))
    }

    /// Move left relative to current position and orientation.
    #[inline]
    pub fn move_left(&mut self, amount: f32) -> &mut Self {
        self.append_translation(Vector3::new(-amount, 0.0, 0.0))
    }

    /// Move up relative to current position and orientation.
    #[inline]
    pub fn move_up(&mut self, amount: f32) -> &mut Self {
        self.append_translation(Vector3::new(0.0, amount, 0.0))
    }

    /// Move down relative to current position and orientation.
    #[inline]
    pub fn move_down(&mut self, amount: f32) -> &mut Self {
        self.append_translation(Vector3::new(0.0, -amount, 0.0))
    }

    /// Adds the specified amount to the translation vector's x component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// x axis.
    #[inline]
    pub fn prepend_translation_x(&mut self, amount: f32) -> &mut Self {
        self.isometry.translation.vector.x += amount;
        self
    }

    /// Adds the specified amount to the translation vector's y component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// y axis.
    #[inline]
    pub fn prepend_translation_y(&mut self, amount: f32) -> &mut Self {
        self.isometry.translation.vector.y += amount;
        self
    }

    /// Adds the specified amount to the translation vector's z component.
    /// i.e. move relative to the parent's (or global, if no parent exists)
    /// z axis.
    #[inline]
    pub fn prepend_translation_z(&mut self, amount: f32) -> &mut Self {
        self.isometry.translation.vector.z += amount;
        self
    }

    /// Sets the translation vector's x component to the specified value.
    #[inline]
    pub fn set_translation_x(&mut self, value: f32) -> &mut Self {
        self.isometry.translation.vector.x = value;
        self
    }

    /// Sets the translation vector's y component to the specified value.
    #[inline]
    pub fn set_translation_y(&mut self, value: f32) -> &mut Self {
        self.isometry.translation.vector.y = value;
        self
    }

    /// Sets the translation vector's z component to the specified value.
    #[inline]
    pub fn set_translation_z(&mut self, value: f32) -> &mut Self {
        self.isometry.translation.vector.z = value;
        self
    }

    /// Premultiply a rotation about the x axis, i.e. perform a rotation about
    /// the parent's x axis (or the global x axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_x_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.prepend_rotation(Vector3::x_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the x axis, i.e. perform a rotation about
    /// the *local* x-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_x_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.append_rotation(Vector3::x_axis(), delta_angle)
    }

    /// Set the rotation about the parent's x axis (or the global x axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_x_axis(&mut self, angle: f32) -> &mut Self {
        self.set_rotation_euler(angle, 0.0, 0.0)
    }

    /// Premultiply a rotation about the y axis, i.e. perform a rotation about
    /// the parent's y axis (or the global y axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_y_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.prepend_rotation(Vector3::y_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the y axis, i.e. perform a rotation about
    /// the *local* y-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_y_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.append_rotation(Vector3::y_axis(), delta_angle)
    }

    /// Set the rotation about the parent's y axis (or the global y axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_y_axis(&mut self, angle: f32) -> &mut Self {
        self.set_rotation_euler(0.0, angle, 0.0)
    }

    /// Premultiply a rotation about the z axis, i.e. perform a rotation about
    /// the parent's z axis (or the global z axis if no parent exists).
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation_z_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.prepend_rotation(-Vector3::z_axis(), delta_angle)
    }

    /// Postmultiply a rotation about the z axis, i.e. perform a rotation about
    /// the *local* z-axis, including any prior rotations that have been performed.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation_z_axis(&mut self, delta_angle: f32) -> &mut Self {
        self.append_rotation(-Vector3::z_axis(), delta_angle)
    }

    /// Set the rotation about the parent's z axis (or the global z axis
    /// if no parent exists). This will *clear any other rotations that have
    /// previously been performed*!
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_z_axis(&mut self, angle: f32) -> &mut Self {
        self.set_rotation_euler(0.0, 0.0, angle)
    }

    /// Perform a rotation about the axis perpendicular to X and Y,
    /// i.e. the most common way to rotate an object in a 2d game.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn rotate_2d(&mut self, delta_angle: f32) -> &mut Self {
        self.prepend_rotation_z_axis(delta_angle)
    }

    /// Set the rotation about the axis perpendicular to X and Y,
    /// i.e. the most common way to rotate an object in a 2d game.
    ///
    /// `angle` is specified in radians.
    #[inline]
    pub fn set_rotation_2d(&mut self, angle: f32) -> &mut Self {
        self.set_rotation_euler(0.0, 0.0, angle)
    }

    /// Premultiply a rotation, i.e. rotate relatively to the parent's orientation
    /// (or the global orientation if no parent exists), about a specified axis.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn prepend_rotation(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        let q = UnitQuaternion::from_axis_angle(&axis, angle);
        self.isometry.rotation = q * self.isometry.rotation;
        self
    }

    /// Postmultiply a rotation, i.e. rotate relatively to the local orientation (the
    /// currently applied rotations), about a specified axis.
    ///
    /// `delta_angle` is specified in radians.
    #[inline]
    pub fn append_rotation(&mut self, axis: Unit<Vector3<f32>>, angle: f32) -> &mut Self {
        self.isometry.rotation *= UnitQuaternion::from_axis_angle(&axis, angle);
        self
    }

    /// Set the position.
    pub fn set_translation<N: RealField + SubsetOf<f32>>(
        &mut self,
        position: Vector3<N>,
    ) -> &mut Self {
        self.isometry.translation.vector = na::convert(position);
        self
    }

    /// Adds the specified amounts to the translation vector.
    pub fn append_translation_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.append_translation(Vector3::new(x, y, z));
        self
    }

    /// Sets the specified values of the translation vector.
    pub fn set_translation_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.set_translation(Vector3::new(x, y, z))
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation<N: RealField + SubsetOf<f32>>(
        &mut self,
        rotation: UnitQuaternion<N>,
    ) -> &mut Self {
        self.isometry.rotation = na::convert(rotation);
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
    /// # use amethyst::core::transform::Transform;
    /// let mut transform = Transform::default();
    ///
    /// transform.set_rotation_euler(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(transform.rotation().euler_angles().0, 1.0);
    /// ```
    pub fn set_rotation_euler(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
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
    pub fn euler_angles(&self) -> (f32, f32, f32) {
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
            .all(|f| f32::is_finite(*f))
    }
    /// Calculates the inverse of this transform, which is in effect the 'view matrix' as
    /// commonly seen in computer graphics. This function computes the view matrix for ONLY
    /// the local transformation, and ignores any `Parent`s of this entity.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let inv_scale = Vector3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        self.isometry
            .inverse()
            .to_homogeneous()
            .append_nonuniform_scaling(&inv_scale)
    }

    /// Calculates the inverse of this transform, which is in effect the 'view matrix' as
    /// commonly seen in computer graphics. This function computes the view matrix for the
    /// global transformation of the entity, and so takes into account `Parent`s.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn global_view_matrix(&self) -> Matrix4<f32> {
        let mut res = self.global_matrix;

        // Perform an in-place inversion of the 3x3 matrix
        {
            let mut slice3x3 = res.fixed_slice_mut::<na::U3, na::U3>(0, 0);
            assert!(slice3x3.try_inverse_mut());
        }

        let mut translation = -res.column(3).xyz();
        translation = res.fixed_slice::<na::U3, na::U3>(0, 0) * translation;

        let mut res_trans = res.column_mut(3);
        res_trans.x = translation.x;
        res_trans.y = translation.y;
        res_trans.z = translation.z;

        res
    }

    /// This function allows for test cases of copying the local matrix to the global matrix.
    /// Useful for tests or other debug type access.
    #[inline]
    pub fn copy_local_to_global(&mut self) {
        self.global_matrix = self.matrix()
    }
}

impl Default for Transform {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            isometry: Isometry3::identity(),
            scale: Vector3::from_element(1.0),
            global_matrix: na::one(),
            parent_matrix: na::one(),
        }
    }
}

register_component_type!(Transform);

/// Creates a Transform using the `Vector3` as the translation vector.
///
/// ```
/// # use amethyst::core::{transform::Transform};
/// # use amethyst::core::math::Vector3;
/// let transform = Transform::from(Vector3::new(100.0, 200.0, 300.0));
/// assert_eq!(transform.translation().x, 100.0);
/// ```
impl From<Vector3<f32>> for Transform {
    fn from(translation: Vector3<f32>) -> Self {
        Transform {
            isometry: Isometry3::new(translation, na::zero()),
            ..Default::default()
        }
    }
}
/// Creates a Transform using the `Vector3<f64>` as the translation vector.
/// Provided for convinience when providing constants.
/// ```
/// # use amethyst::core::transform::Transform;
/// # use amethyst::core::math::Vector3;
/// let transform = Transform::from(Vector3::new(100.0, 200.0, 300.0));
/// assert_eq!(transform.translation().x, 100.0);
impl From<Vector3<f64>> for Transform {
    #[inline]
    fn from(translation: Vector3<f64>) -> Self {
        Transform {
            isometry: Isometry3::new(na::convert(translation), na::zero()),
            ..Default::default()
        }
    }
}

/// Format for prefab Transform serialization
#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid, SerdeDiff)]
#[uuid = "f062a20b-250f-44b3-a58a-4a00f7692c22"]
#[serde(rename = "Transform", default)]
pub struct TransformValues {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl TransformValues {
    /// Initialize a new TransformValues object that can later be use to generate a `Transform`
    pub fn new(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }
}

impl Default for TransformValues {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        TransformValues {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }
}

impl From<TransformValues> for Transform {
    fn from(transform_values: TransformValues) -> Self {
        let TransformValues {
            translation,
            rotation,
            scale,
        } = transform_values;

        let isometry = Isometry3::from_parts(
            Translation3::new(translation[0], translation[1], translation[2]),
            Unit::new_normalize(Quaternion::new(
                rotation[3],
                rotation[0],
                rotation[1],
                rotation[2],
            )),
        );
        let scale = Vector3::new(scale[0], scale[1], scale[2]);

        Transform {
            isometry,
            scale,
            ..Default::default()
        }
    }
}

impl From<Transform> for TransformValues {
    fn from(t: Transform) -> Self {
        TransformValues {
            translation: t.isometry.translation.vector.into(),
            rotation: t.isometry.rotation.as_ref().coords.into(),
            scale: t.scale.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        approx::*,
        math::{UnitQuaternion, Vector3},
        transform::Transform,
    };

    /// Sanity test for concat operation
    #[test]
    fn test_mul() {
        // For the condition to hold both scales must be uniform
        let mut first = Transform::default();
        first.set_translation_xyz(20., 10., -3.);
        first.set_scale(Vector3::new(2.0, 2.0, 2.0));
        first.set_rotation(
            UnitQuaternion::rotation_between(
                &Vector3::new(-1.0, 1.0, 2.0),
                &Vector3::new(1.0, 0.0, 0.0),
            )
            .unwrap(),
        );

        let mut second = Transform::default();
        second.set_translation_xyz(2., 1., -3.);
        second.set_scale(Vector3::new(1.0, 1.0, 1.0));
        second.set_rotation(
            UnitQuaternion::rotation_between(
                &Vector3::new(7.0, -1.0, 3.0),
                &Vector3::new(2.0, 1.0, 1.0),
            )
            .unwrap(),
        );

        // check Mat(first * second) == Mat(first) * Mat(second)
        assert_relative_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
            max_relative = 0.000_001,
        );
        assert_relative_eq!(
            first.matrix() * second.matrix(),
            first.concat(&second).matrix(),
            max_relative = 0.000_001,
        );
    }

    /// Test correctness of the view matrix locally
    #[test]
    fn test_view_matrix() {
        let mut transform = Transform::default();
        transform.set_translation_xyz(5.0, 70.1, 43.7);
        transform.set_scale(Vector3::new(1.0, 5.0, 8.9));
        transform.set_rotation(
            UnitQuaternion::rotation_between(
                &Vector3::new(-1.0, 1.0, 2.0),
                &Vector3::new(1.0, 0.0, 0.0),
            )
            .unwrap(),
        );

        assert_ulps_eq!(
            transform.matrix().try_inverse().unwrap(),
            transform.view_matrix(),
        );
    }

    /// Test correctness of global view matrix vs. inverse matrix globally
    #[test]
    fn test_global_view_matrix() {
        let mut transform = Transform::default();
        transform.set_translation_xyz(5.0, 70.1, 43.7);
        transform.set_scale(Vector3::new(1.0, 5.0, 8.9));
        transform.set_rotation(
            UnitQuaternion::rotation_between(
                &Vector3::new(-1.0, 1.0, 2.0),
                &Vector3::new(1.0, 0.0, 0.0),
            )
            .unwrap(),
        );

        assert_ulps_eq!(
            transform.global_matrix().try_inverse().unwrap(),
            transform.global_view_matrix(),
        );
    }

    #[test]
    fn ser_deser() {
        let mut transform = Transform::default();
        transform.set_translation_xyz(1.0, 2.0, 3.0);
        transform.set_scale(Vector3::new(4.0, 5.0, 6.0));
        transform.set_rotation(
            UnitQuaternion::rotation_between(
                &Vector3::new(-1.0, 1.0, 2.0),
                &Vector3::new(1.0, 0.0, 0.0),
            )
            .unwrap(),
        );
        let s: String =
            ron::ser::to_string_pretty(&transform, ron::ser::PrettyConfig::default()).unwrap();
        let transform2: Transform = ron::de::from_str(&s).unwrap();

        assert_eq!(transform, transform2);
    }

    #[test]
    fn deser_seq_default_identity() {
        let transform: Transform = ron::de::from_str("()").unwrap();
        assert_eq!(transform, Transform::default());
    }

    #[test]
    fn is_finite() {
        let mut transform = Transform::default();
        assert!(transform.is_finite());

        transform.global_matrix.fill_row(2, std::f32::NAN);
        assert!(!transform.is_finite());
    }
}
