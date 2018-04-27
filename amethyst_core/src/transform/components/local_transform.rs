//! Local transform component.

use cgmath::{Basis2, Deg, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4,
             One, Point2, Point3, Quaternion, Rotation, Rotation2, Rotation3,
             Transform as CgTransform, Vector2, Vector3, Rad, Zero, Angle, Decomposed};
use orientation::Orientation;
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};
use std::{cmp, ops};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are preformed in this order: scale, then rotation, then translation,
/// or in maths `world_vertex = translation * rotation * scale * model_vertex`
#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub inner: Decomposed<Vector3<f32>, Quaternion<f32>>
}

impl Transform {
    /// Create a new transform from an existing position, rotation, and scale factor.
    ///
    /// If you only want to set one of the parameters it's probably better to use the `with_`
    /// methods, for example
    ///
    /// ```
    /// # use amethyst_core::Transform;
    /// # use amethyst_core::cgmath::Vector3;
    /// let t = Transform::default()
    ///     .with_position(Vector3 { x: 0., y: 0., z: 1. });
    /// ```
    #[inline]
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: f32) -> Transform {
        Transform {
            inner: Decomposed {
                disp: position,
                rot: rotation,
                scale
            }
        }
    }


    /// Get the current position
    #[inline]
    pub fn position(&self) -> Vector3<f32> {
        self.inner.disp
    }

    /// Get the current rotation
    #[inline]
    pub fn rotation(&self) -> Quaternion<f32> {
        self.inner.rot
    }

    /// Get the current rotation
    #[inline]
    pub fn scale(&self) -> f32 {
        self.inner.scale
    }

    /// Set the position.
    #[inline]
    pub fn set_position<P>(&mut self, position: P) -> &mut Self
        where P: Into<Vector3<f32>>
    {
        self.inner.disp = position.into();
        self
    }

    /// Set the rotation
    #[inline]
    pub fn set_rotation<Q>(&mut self, rotation: Q) -> &mut Self
        where Q: Into<Quaternion<f32>>
    {
        self.inner.rot = rotation.into().normalize();
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis. Also known at the pitch.
    ///  - y - The angle to apply around the y axis. Also known at the yaw.
    ///  - z - The angle to apply around the z axis. Also known at the roll.
    #[inline]
    pub fn set_rotation_euler<A>(&mut self, x: A, y: A, z: A) -> &mut Self
        where A: Angle<Unitless=f32>,
              Rad<f32>: From<A>
    {
        // we use Euler as an internediate stage to avoid gimbal lock
        self.inner.rot = Quaternion::from(Euler { x, y, z });
        self
    }

    /// Set the scale
    #[inline]
    pub fn set_scale(&mut self, scale: f32) -> &mut Self {
        self.inner.scale = scale;
        self
    }

    /// Set the position.
    #[inline]
    pub fn with_position<P>(mut self, position: P) -> Self
        where P: Into<Vector3<f32>>
    {
        self.inner.disp = position.into();
        self
    }

    /// Set the rotation
    #[inline]
    pub fn with_rotation<Q>(mut self, rotation: Q) -> Self
        where Q: Into<Quaternion<f32>>
    {
        self.inner.rot = rotation.into().normalize();
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis. Also known at the pitch.
    ///  - y - The angle to apply around the y axis. Also known at the yaw.
    ///  - z - The angle to apply around the z axis. Also known at the roll.
    #[inline]
    pub fn with_rotation_euler<A>(mut self, x: A, y: A, z: A) -> Self
        where A: Angle<Unitless=f32>,
              Rad<f32>: From<A>
    {
        // we use Euler as an internediate stage to avoid gimbal lock
        self.inner.rot = Quaternion::from(Euler { x, y, z });
        self
    }

    /// Set the scale
    #[inline]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.inner.scale = scale;
        self
    }

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
    #[inline]
    pub fn look_at(&mut self, position: Point3<f32>, up: Vector3<f32>) -> &mut Self {
        self.inner = Decomposed::look_at(Point3::from_vec(self.inner.disp), position, up);
        self
    }

    /// Convert this transform's rotation into an Orientation, guaranteed to be 3 unit orthogonal
    /// vectors
    #[inline]
    pub fn orientation(&self) -> Orientation {
        Orientation::from(Matrix3::from(self.inner.rot))
    }

    /// Move relatively to its current position.
    #[inline]
    pub fn move_global(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.inner.disp += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// Equivalent to rotating the translation before applying.
    #[inline]
    pub fn move_local(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.inner.disp += self.inner.rot * translation;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_global(&mut self, direction: Vector3<f32>, distance: f32) -> &mut Self {
        if !ulps_eq!(direction, Zero::zero()) {
            self.inner.disp += direction.normalize_to(distance);
        }
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Vector3<f32>, distance: f32) -> &mut Self {
        if !ulps_eq!(direction, Zero::zero()) {
            self.inner.disp += self.inner.rot * direction.normalize_to(distance);
        }
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
    pub fn pitch_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(Vector3::unit_x(), angle)
    }

    /// Pitch relatively to its own rotation.
    #[inline]
    pub fn pitch_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(Vector3::unit_x(), angle)
    }

    /// Yaw relatively to the world.
    #[inline]
    pub fn yaw_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(Vector3::unit_y(), angle)
    }

    /// Yaw relatively to its own rotation.
    #[inline]
    pub fn yaw_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(Vector3::unit_y(), angle)
    }

    /// Roll relatively to the world.
    #[inline]
    pub fn roll_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(-Vector3::unit_z(), angle)
    }

    /// Roll relatively to its own rotation.
    #[inline]
    pub fn roll_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(-Vector3::unit_z(), angle)
    }

    /// Rotate relatively to the world
    #[inline]
    pub fn rotate_global<A: Into<Rad<f32>>>(&mut self, axis: Vector3<f32>, angle: A) -> &mut Self {
        let mag = axis.magnitude();
        debug_assert!(
            !ulps_eq!(mag, Zero::zero()),
            "Axis of rotation must not be zero"
        );
        // avoid doing sqrt twice by reusing mag
        let q = Quaternion::from_axis_angle(axis / mag, angle);
        self.inner.rot = q * self.inner.rot;
        self
    }

    /// Rotate relatively to the current orientation
    #[inline]
    pub fn rotate_local<A: Into<Rad<f32>>>(&mut self, axis: Vector3<f32>, angle: A) -> &mut Self {
        let mag = axis.magnitude();
        debug_assert!(
            !ulps_eq!(mag, Zero::zero()),
            "Axis of rotation must not be zero"
        );
        // avoid doing sqrt twice by reusing mag
        let q = Quaternion::from_axis_angle(axis / mag, angle);
        self.inner.rot = self.inner.rot * q;
        self
    }

    /// Calculates the inverse of this transform, which we need to render.
    ///
    /// We can exploit the extra information we have to perform this inverse faster than `O(n^3)`.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        // todo
        use cgmath::SquareMatrix;
        self.matrix().invert().unwrap()
    }

    /// Create a new `Transform`.
    ///
    /// This transform performs no rotation, translation, or scaling by default. The transform
    /// that does nothing is known as the *identity* transform.
    ///
    /// This is a convenience method calling Default::default()
    pub fn identity() -> Self {
        Default::default()
    }

    /// Get the matrix representation of this transform
    ///
    /// ```rust
    /// // This is a convenience method wrapping `impl From<Matrix4<f32> for Transform>`
    /// let t = Transform::default();
    /// assert_eq!(t.matrix(), Matrix4::from(t));
    /// ```
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from(self.clone())
    }

    /// Get the 2d rotation basis
    fn basis_2d(&self) -> Basis2<f32> {
        Rotation2::from_angle(-Euler::from(self.inner.rot).z)
    }
}

impl From<Transform> for Matrix4<f32> {
    fn from(t: Transform) -> Self {
        t.inner.into()
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            inner: Decomposed {
                scale: One::one(),
                rot: Quaternion::one(),
                disp: Zero::zero()
            }
        }
    }
}

impl ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        <Self as CgTransform<Point3<f32>>>::concat(&self, &rhs)
    }
}

impl cmp::PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.inner.scale == other.inner.scale
            && self.inner.rot == other.inner.rot
            && self.inner.disp == other.inner.disp
    }
}

impl One for Transform {
    fn one() -> Transform {
        Default::default()
    }
}

impl CgTransform<Point3<f32>> for Transform {
    fn one() -> Self {
        One::one()
    }

    fn look_at(eye: Point3<f32>, center: Point3<f32>, up: Vector3<f32>) -> Self {
        Transform { inner: Decomposed::look_at(eye, center, up) }
    }

    fn transform_vector(&self, vec: Vector3<f32>) -> Vector3<f32> {
        self.inner.transform_vector(vec)
    }

    fn inverse_transform_vector(&self, vec: Vector3<f32>) -> Option<Vector3<f32>> {
        self.inner.inverse_transform_vector(vec)
    }

    fn transform_point(&self, point: Point3<f32>) -> Point3<f32> {
        self.inner.transform_point(point)
    }

    fn concat(&self, other: &Self) -> Self {
        Transform { inner: self.inner.concat(&other.inner) }
    }

    fn inverse_transform(&self) -> Option<Self> {
        Some(Transform { inner: self.inner.inverse_transform()? })
    }
}

impl CgTransform<Point2<f32>> for Transform {
    fn one() -> Self {
        One::one()
    }

    fn look_at(_eye: Point2<f32>, _center: Point2<f32>, _up: Vector2<f32>) -> Self {
        panic!("Can't compute look at for 2D")
    }

    fn transform_vector(&self, vec: Vector2<f32>) -> Vector2<f32> {
        self.basis_2d().rotate_vector(vec * self.inner.scale)
    }

    fn inverse_transform_vector(&self, vec: Vector2<f32>) -> Option<Vector2<f32>> {
        if ulps_eq!(self.inner.scale, Zero::zero()) {
            None
        } else {
            Some(self.basis_2d().rotate_vector(vec / self.inner.scale))
        }
    }

    fn transform_point(&self, point: Point2<f32>) -> Point2<f32> {
        self.basis_2d().rotate_point(point * self.inner.scale) + self.inner.disp.truncate()
    }

    fn concat(&self, other: &Self) -> Self {
        <Self as CgTransform<Point3<f32>>>::concat(self, other)
    }

    fn inverse_transform(&self) -> Option<Self> {
        <Self as CgTransform<Point3<f32>>>::inverse_transform(self)
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

