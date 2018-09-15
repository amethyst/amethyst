//! Local transform component.

use cgmath::{
    Angle, Array, Basis2, Deg, ElementWise, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4,
    One, Point2, Point3, Quaternion, Rad, Rotation, Rotation2, Rotation3, Transform as CgTransform,
    Vector2, Vector3, Zero,
};
use orientation::Orientation;
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
///
/// The transforms are preformed in this order: scale, then rotation, then translation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Transform {
    /// Quaternion [w (scalar), x, y, z]
    pub rotation: Quaternion<f32>,
    /// Scale vector [x, y, z]
    pub scale: Vector3<f32>,
    /// Translation/position vector [x, y, z]
    pub translation: Vector3<f32>,
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
    #[inline]
    pub fn look_at(&mut self, position: Point3<f32>, up: Vector3<f32>) -> &mut Self {
        self.rotation = Quaternion::look_at(Point3::from_vec(self.translation) - position, up);
        // Catch NaNs etc. in debug mode.
        debug_assert!(
            self.rotation.s.is_finite()
                && self.rotation.v.x.is_finite()
                && self.rotation.v.y.is_finite()
                && self.rotation.v.z.is_finite(),
            "`look_at` should be finite to be useful"
        );
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
        let quat: Matrix3<f32> = self.rotation.into();
        // This should probably be in cgmath eventually.
        //
        // Note: Not benchmarked

        Matrix4 {
            x: (quat.x * self.scale.x).extend(0.),
            y: (quat.y * self.scale.y).extend(0.),
            z: (quat.z * self.scale.z).extend(0.),
            w: self.translation.extend(1.0),
        }
    }

    /// Convert this transform's rotation into an Orientation, guaranteed to be 3 unit orthogonal
    /// vectors
    pub fn orientation(&self) -> Orientation {
        Orientation::from(Matrix3::from(self.rotation))
    }

    /// Move relatively to its current position.
    #[inline]
    pub fn move_global(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.translation += translation;
        self
    }

    /// Move relatively to its current position and orientation.
    ///
    /// Equivalent to rotating the translation before applying.
    #[inline]
    pub fn move_local(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.translation += self.rotation * translation;
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_global(&mut self, direction: Vector3<f32>, distance: f32) -> &mut Self {
        if !ulps_eq!(direction, Zero::zero()) {
            self.translation += direction.normalize_to(distance);
        }
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Vector3<f32>, distance: f32) -> &mut Self {
        if !ulps_eq!(direction, Zero::zero()) {
            self.translation += self.rotation * direction.normalize_to(distance);
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
        debug_assert!(
            !ulps_eq!(axis.magnitude2(), Zero::zero()),
            "Axis of rotation must not be zero"
        );
        let q = Quaternion::from_axis_angle(axis.normalize(), angle);
        self.rotation = q * self.rotation;
        self
    }

    /// Rotate relatively to the current orientation
    #[inline]
    pub fn rotate_local<A: Into<Rad<f32>>>(&mut self, axis: Vector3<f32>, angle: A) -> &mut Self {
        debug_assert!(
            !ulps_eq!(axis.magnitude2(), Zero::zero()),
            "Axis of rotation must not be zero"
        );
        let q = Quaternion::from_axis_angle(axis.normalize(), angle);
        self.rotation = self.rotation * q;
        self
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.translation = position;
        self
    }

    /// Set the rotation using Euler x, y, z.
    ///
    /// # Arguments
    ///
    ///  - x - The angle to apply around the x axis. Also known as the pitch.
    ///  - y - The angle to apply around the y axis. Also known as the yaw.
    ///  - z - The angle to apply around the z axis. Also known as the roll.
    pub fn set_rotation<A>(&mut self, x: A, y: A, z: A) -> &mut Self
    where
        A: Angle<Unitless = f32>,
        Rad<f32>: From<A>,
    {
        // we use Euler as an internediate stage to avoid gimbal lock
        self.rotation = Quaternion::from(Euler { x, y, z });
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
}

impl Default for Transform {
    /// The default transform does nothing when used to transform an entity.
    fn default() -> Self {
        Transform {
            translation: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::from_value(1.),
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
            translation,
            ..Default::default()
        }
    }
}

impl CgTransform<Point3<f32>> for Transform {
    fn one() -> Self {
        Default::default()
    }

    fn look_at(eye: Point3<f32>, center: Point3<f32>, up: Vector3<f32>) -> Self {
        let rotation = Quaternion::look_at(center - eye, up);
        let translation = rotation.rotate_vector(Point3::origin() - eye);
        let scale = Vector3::from_value(1.);
        Self {
            scale,
            rotation,
            translation,
        }
    }

    fn transform_vector(&self, vec: Vector3<f32>) -> Vector3<f32> {
        self.rotation
            .rotate_vector(vec.mul_element_wise(self.scale))
    }

    fn inverse_transform_vector(&self, vec: Vector3<f32>) -> Option<Vector3<f32>> {
        if ulps_eq!(self.scale, &Vector3::zero()) {
            None
        } else {
            Some(
                self.rotation
                    .invert()
                    .rotate_vector(vec.div_element_wise(self.scale)),
            )
        }
    }

    fn transform_point(&self, point: Point3<f32>) -> Point3<f32> {
        let p = Point3::from_vec(point.to_vec().mul_element_wise(self.scale));
        self.rotation.rotate_point(p) + self.translation
    }

    fn concat(&self, other: &Self) -> Self {
        Self {
            scale: self.scale.mul_element_wise(other.scale),
            rotation: self.rotation * other.rotation,
            translation: self
                .rotation
                .rotate_vector(other.translation.mul_element_wise(self.scale))
                + self.translation,
        }
    }

    fn inverse_transform(&self) -> Option<Self> {
        if ulps_eq!(self.scale, Vector3::zero()) {
            None
        } else {
            let scale = 1. / self.scale;
            let rotation = self.rotation.invert();
            let translation = rotation
                .rotate_vector(self.translation)
                .mul_element_wise(-scale);
            Some(Self {
                translation,
                rotation,
                scale,
            })
        }
    }
}

impl CgTransform<Point2<f32>> for Transform {
    fn one() -> Self {
        Default::default()
    }

    fn look_at(_eye: Point2<f32>, _center: Point2<f32>, _up: Vector2<f32>) -> Self {
        panic!("Can't compute look at for 2D")
    }

    fn transform_vector(&self, vec: Vector2<f32>) -> Vector2<f32> {
        let rot: Basis2<f32> = Rotation2::from_angle(-Euler::from(self.rotation).z);
        rot.rotate_vector(vec.mul_element_wise(self.scale.truncate()))
    }

    fn inverse_transform_vector(&self, vec: Vector2<f32>) -> Option<Vector2<f32>> {
        if ulps_eq!(self.scale, &Vector3::zero()) {
            None
        } else {
            let rot: Basis2<f32> = Rotation2::from_angle(-Euler::from(self.rotation).z);
            Some(rot.rotate_vector(vec.div_element_wise(self.scale.truncate())))
        }
    }

    fn transform_point(&self, point: Point2<f32>) -> Point2<f32> {
        let p = Point2::from_vec(point.to_vec().mul_element_wise(self.scale.truncate()));
        let rot: Basis2<f32> = Rotation2::from_angle(-Euler::from(self.rotation).z);
        rot.rotate_point(p) + self.translation.truncate()
    }

    fn concat(&self, other: &Self) -> Self {
        Self {
            scale: self.scale.mul_element_wise(other.scale),
            rotation: self.rotation * other.rotation,
            translation: self
                .rotation
                .rotate_vector(other.translation.mul_element_wise(self.scale))
                + self.translation,
        }
    }

    fn inverse_transform(&self) -> Option<Self> {
        if ulps_eq!(self.scale, Vector3::zero()) {
            None
        } else {
            let scale = 1. / self.scale;
            let rotation = self.rotation.invert();
            let translation = rotation
                .rotate_vector(self.translation)
                .mul_element_wise(-scale);
            Some(Self {
                translation,
                rotation,
                scale,
            })
        }
    }
}

/// Sanity test for concat operation
#[test]
fn test_mul() {
    // For the condition to hold both scales must be uniform
    let first = Transform {
        rotation: Quaternion::look_at(Vector3::new(-1., 1., 2.), Vector3::new(1., 0., 0.)),
        translation: Vector3::new(20., 10., -3.),
        scale: Vector3::new(2., 2., 2.),
    };
    let second = Transform {
        rotation: Quaternion::look_at(Vector3::new(7., -1., 3.), Vector3::new(2., 1., 1.)),
        translation: Vector3::new(2., 1., -3.),
        scale: Vector3::new(1., 1., 1.),
    };
    // check Mat(first * second) == Mat(first) * Mat(second)
    assert_ulps_eq!(
        first.matrix() * second.matrix(),
        <Transform as CgTransform<Point3<f32>>>::concat(&first, &second).matrix()
    );
    assert_ulps_eq!(
        first.matrix() * second.matrix(),
        <Transform as CgTransform<Point2<f32>>>::concat(&first, &second).matrix()
    );
}
