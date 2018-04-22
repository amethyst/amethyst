//! Local transform component.

use cgmath::{Array, Basis2, Deg, ElementWise, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4,
             One, Point2, Point3, Quaternion, Rotation, Rotation2, Rotation3,
             Transform as CgTransform, Vector2, Vector3, Vector4, Rad, Zero};
use orientation::Orientation;
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Quaternion [w (scalar), x, y, z]
    pub rotation: Quaternion<f32>,
    /// Scale vector [x, y, z]
    pub scale: Vector3<f32>,
    /// Translation/position vector [x, y, z]
    pub translation: Vector3<f32>,
}

impl Transform {
    /// Rotate to look at a point in space (without rolling)
    // Does this make sense, because the position doesn't take into account parent transformations?
    #[inline]
    pub fn look_at(&mut self, up: Vector3<f32>, position: Point3<f32>) -> &mut Self {
        self.rotation = Quaternion::look_at(position - Point3::from_vec(self.translation), up);
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
        // multiplying a general matrix by a diagonal matrix is equivalent to multiplying each row
        // of the general matrix with the corresponding value from the diagonal matrix (see
        // http://www.solitaryroad.com/c108.html for example). If we do this manually we can cut
        // down the number of arithmetic operations and speed up stuff.
        //
        // This should probably be in cgmath eventually.
        //
        // Note: Not benchmarked
        let x = Vector4 {
            x: quat.x.x * self.scale.x,
            y: quat.x.y * self.scale.y,
            z: quat.x.z * self.scale.z,
            w: 0.0
        };
        let y = Vector4 {
            x: quat.y.x * self.scale.x,
            y: quat.y.y * self.scale.y,
            z: quat.y.z * self.scale.z,
            w: 0.0
        };
        let z = Vector4 {
            x: quat.z.x * self.scale.x,
            y: quat.z.y * self.scale.y,
            z: quat.z.z * self.scale.z,
            w: 0.0
        };

        let mat = Matrix4 { x, y, z, w: self.translation.extend(1.0)};
        mat
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
            self.translation += direction.normalize() * distance;
        }
        self
    }

    /// Move a distance along an axis.
    ///
    /// It will not move in the case where the axis is zero, for any distance.
    #[inline]
    pub fn move_along_local(&mut self, direction: Vector3<f32>, distance: f32) -> &mut Self {
        if !ulps_eq!(direction, Zero::zero()) {
            self.translation += self.rotation * direction.normalize() * distance;
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
    pub fn set_rotation<A: Into<Rad<f32>>>(&mut self, x: A, y: A, z: A) -> &mut Self {
        self.rotation = Quaternion::from_angle_x(x) * Quaternion::from_angle_y(y)
            * Quaternion::from_angle_z(z);
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::from_value(1.),
        }
    }
}

impl Transform {
    /// Create a new `Transform`.
    ///
    /// If you call `matrix` on this, then you would get an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl CgTransform<Point3<f32>> for Transform {
    fn one() -> Self {
        Default::default()
    }

    fn look_at(eye: Point3<f32>, center: Point3<f32>, up: Vector3<f32>) -> Self {
        let rotation = Quaternion::look_at(center - eye, up);
        let translation = rotation.rotate_vector(Point3::origin() - eye);
        Self {
            scale: Vector3::from_value(1.),
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
            translation: self.rotation
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
            translation: self.rotation
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
