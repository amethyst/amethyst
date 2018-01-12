//! Local transform component.

use cgmath::{Array, Deg, ElementWise, EuclideanSpace, InnerSpace, Matrix3, Matrix4, One, Point3,
             Quaternion, Rotation, Rotation3, SquareMatrix, Transform, Vector3, Zero};
use orientation::Orientation;
use specs::{Component, DenseVecStorage, FlaggedStorage};

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    /// Quaternion [w (scalar), x, y, z]
    pub rotation: Quaternion<f32>,
    /// Scale vector [x, y, z]
    pub scale: Vector3<f32>,
    /// Translation/position vector [x, y, z]
    pub translation: Vector3<f32>,
}

impl LocalTransform {
    /// Rotate to look at a point in space (without rolling)
    pub fn look_at(&mut self, orientation: &Orientation, position: Point3<f32>) -> &mut Self {
        self.rotation = Quaternion::look_at(
            position - Point3::from_vec(self.translation),
            orientation.up.into(),
        ).into();
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's global `Transform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        let quat: Matrix3<f32> = Quaternion::from(self.rotation).into();
        let scale: Matrix3<f32> = Matrix3::from_diagonal(self.scale);
        let mut matrix: Matrix4<f32> = (&quat * scale).into();
        matrix.w = self.translation.extend(1.0f32);
        matrix
    }

    /// Move relatively to its current position and orientation.
    pub fn move_forward(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.forward.into(), amount)
    }

    /// Move relatively to its current position, but independently from its orientation.
    /// Ideally, first normalize the direction and then multiply it
    /// by whatever amount you want to move before passing the vector to this method
    #[inline]
    pub fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self {
        self.translation = self.translation + direction;
        self
    }

    /// Move relatively to its current position and orientation.
    #[inline]
    pub fn move_local(&mut self, axis: Vector3<f32>, amount: f32) -> &mut Self {
        let delta = Quaternion::from(self.rotation).conjugate() * axis.normalize() * amount;

        self.translation = self.translation + delta;
        self
    }

    /// Move relatively to its current position and orientation.
    pub fn move_right(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.right.into(), amount)
    }

    /// Move relatively to its current position and orientation.
    pub fn move_up(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.up.into(), amount)
    }

    /// Pitch relatively to the world.
    pub fn pitch_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.right.into(), angle)
    }

    /// Pitch relatively to its own rotation.
    pub fn pitch_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.right.into(), angle)
    }

    /// Roll relatively to the world.
    pub fn roll_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.forward.into(), angle)
    }

    /// Roll relatively to its own rotation.
    pub fn roll_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.forward.into(), angle)
    }

    /// Add a rotation to the current rotation
    #[inline]
    pub fn rotate(&mut self, quat: Quaternion<f32>) -> &mut Self {
        self.rotation = (quat * Quaternion::from(self.rotation)).into();
        self
    }

    /// Rotate relatively to the world
    #[inline]
    pub fn rotate_global(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let axis_normalized = Vector3::from(axis).normalize();
        let q = Quaternion::from_axis_angle(axis_normalized, angle);

        self.rotate(q)
    }

    /// Rotate relatively to the current orientation
    #[inline]
    pub fn rotate_local(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let rel_axis_normalized = Quaternion::from(self.rotation)
            .rotate_vector(Vector3::from(axis))
            .normalize();
        let q = Quaternion::from_axis_angle(rel_axis_normalized, angle);

        self.rotate(q)
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.translation = position;
        self
    }

    /// Set the rotation using Euler x, y, z.
    pub fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self {
        let rotation = Quaternion::from_angle_x(x.into()) * Quaternion::from_angle_y(y.into())
            * Quaternion::from_angle_z(z.into());

        self.rotation = rotation.into();
        self
    }

    /// Calculate the view matrix from the given data.
    pub fn to_view_matrix(&self, orientation: &Orientation) -> Matrix4<f32> {
        let center = self.translation + orientation.forward;
        Matrix4::look_at(
            Point3::from_vec(self.translation),
            Point3::from_vec(center),
            orientation.up,
        )
    }

    /// Yaw relatively to the world.
    pub fn yaw_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.up.into(), angle)
    }

    /// Yaw relatively to its own rotation.
    pub fn yaw_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.up.into(), angle)
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        LocalTransform {
            translation: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::from_value(1.),
        }
    }
}

impl LocalTransform {
    /// Create a new `LocalTransform`.
    ///
    /// If you call `matrix` on this, then you would get an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for LocalTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Transform<Point3<f32>> for LocalTransform {
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
