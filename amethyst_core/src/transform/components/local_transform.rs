//! Local transform component.

use std;

use cgmath::{Angle, Array, Deg, ElementWise, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4, One, Point3,
             Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Transform as CgTransform, Vector3, Zero};
use orientation::Orientation;
use specs::{Component, DenseVecStorage, FlaggedStorage};
use super::super::{Move, Pitch, Roll, Rotate, Scale, Yaw};

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
    /// Create a new `LocalTransform`.
    ///
    /// If you call `matrix` on this, then you would get an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Transform {
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

    /// Calculate the view matrix from the given data.
    pub fn to_view_matrix(&self, orientation: &Orientation) -> Matrix4<f32> {
        let center = self.translation + orientation.forward;
        Matrix4::look_at(
            Point3::from_vec(self.translation),
            Point3::from_vec(center),
            orientation.up,
        )
    }
}

impl Transform {
    /// Add a rotation to the current rotation
    #[inline]
    pub fn rotate(&mut self, quat: Quaternion<f32>) -> &mut Self {
        self.rotation = (quat * Quaternion::from(self.rotation)).into();
        self
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
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

impl Move for Transform {
    /// Move relatively to its current position and orientation.
    fn move_backward(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.forward.into(), -amount)
    }

    /// Move relatively to its current position and orientation.
    fn move_down(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.up.into(), -amount)
    }

    /// Move relatively to its current position and orientation.
    fn move_forward(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.forward.into(), amount)
    }

    /// Move relatively to its current position, but independently from its orientation.
    /// Ideally, first normalize the direction and then multiply it
    /// by whatever amount you want to move before passing the vector to this method
    #[inline]
    fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self {
        self.translation = self.translation + direction;
        self
    }

    /// Move relatively to its current position and orientation.
    fn move_left(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.right.into(), -amount)
    }

    /// Move relatively to its current position and orientation.
    #[inline]
    fn move_local(&mut self, axis: Vector3<f32>, amount: f32) -> &mut Self {
        let delta = Quaternion::from(self.rotation).conjugate().invert() * axis.normalize() * amount;

        self.translation = self.translation + delta;
        self
    }

    /// Move relatively to its current position and orientation.
    fn move_right(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.right.into(), amount)
    }

    /// Move relatively to its current position and orientation.
    fn move_up(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.up.into(), amount)
    }

    /// Get current position
    fn position(&self) -> Point3<f32> {
        Point3::from_vec(self.translation)
    }

    /// Set the position.
    fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.translation = position.to_vec();
        self
    }
}

impl Pitch for Transform {
    /// Pitch relatively to the world.
    fn pitch_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.right.into(), angle)
    }

    /// Pitch relatively to its own rotation.
    fn pitch_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.right.into(), angle)
    }
}

impl Roll for Transform {
    /// Roll relatively to the world.
    fn roll_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.forward.into(), angle)
    }

    /// Roll relatively to its own rotation.
    fn roll_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.forward.into(), angle)
    }
}

impl Rotate for Transform {
    /// Rotate to look at a point in space (without rolling)
    fn look_at(&mut self, orientation: &Orientation, position: Point3<f32>) -> &mut Self {
        self.rotation = Quaternion::look_at(
            position - Point3::from_vec(self.translation),
            orientation.up.into(),
        ).into();
        self
    }

    /// Rotate whole local translation around a point at center via an axis and an angle
    fn rotate_around(&mut self, center: Point3<f32>, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        self.translation -= center.to_vec();
        self.rotate(Quaternion::from_axis_angle(axis, angle));
        self.translation += center.to_vec();
        self
    }

    /// Rotate relatively to the world
    #[inline]
    fn rotate_global(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let axis_normalized = Vector3::from(axis).normalize();
        let q = Quaternion::from_axis_angle(axis_normalized, angle);

        self.rotate(q)
    }

    /// Rotate relatively to the current orientation
    #[inline]
    fn rotate_local(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let rel_axis_normalized = Quaternion::from(self.rotation)
            .rotate_vector(Vector3::from(axis))
            .normalize();
        let q = Quaternion::from_axis_angle(rel_axis_normalized, angle);

        self.rotate(q)
    }

    /// Get current rotation as x, y, z degree values
    fn rotation(&self) -> (Deg<f32>, Deg<f32>, Deg<f32>) {
        let euler = Euler::from(self.rotation);

        (
            euler.x.into(),
            euler.y.into(),
            euler.z.into(),
        )
    }

    /// Set the rotation using Euler x, y, z.
    fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self
        where D: Angle, Rad<<D as Angle>::Unitless>: std::convert::From<D>
    {
        self.rotation = Quaternion::from(Euler { x, y, z }).cast().unwrap();
        self
    }
}

impl Scale for Transform {
    /// Get current scale as x, y, z values
    fn scale(&self) -> (f32, f32, f32) {
        (self.scale[0], self.scale[1], self.scale[2])
    }

    // Set new scale
    fn set_scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.scale[0] = x;
        self.scale[1] = y;
        self.scale[2] = z;

        self
    }
}

impl Yaw for Transform {
    /// Yaw relatively to the world.
    fn yaw_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.up.into(), angle)
    }

    /// Yaw relatively to its own rotation.
    fn yaw_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.up.into(), angle)
    }
}

#[test]
fn position() {
    // Set and get position
    let mut trans = Transform::default();
    let pos = trans
        .set_position([50., 0., 0.].into())
        .position()
    ;

    assert_eq!(pos[0], 50.);
    assert_eq!(pos[1], 0.);
    assert_eq!(pos[2], 0.);
}

#[test]
fn movement() {
    // Move local and global produce same, correct results for default
    let mut trans1 = Transform::default();
    let mut trans2 = Transform::default();

    let pos1 = trans1
        .move_global([50., 0., 0.].into())
        .position()
    ;

    let pos2 = trans2
        .move_local([1., 0., 0.].into(), 50.)
        .position()
    ;

    assert_eq!(pos1.x, 50.);
    assert_eq!(pos1.y, 0.);
    assert_eq!(pos1.z, 0.);
    assert_eq!(pos2.x, 50.);
    assert_eq!(pos2.y, 0.);
    assert_eq!(pos2.z, 0.);
}

#[test]
fn simple_rotation() {
    // Set and get rotation
    let mut trans = Transform::default();
    let rot = trans
        .set_rotation(Deg(10.), Deg(20.), Deg(30.))
        .rotation()
    ;

    println!("{:?}", rot);
    assert_eq!(f32::round((rot.0).0), 10.);
    assert_eq!(f32::round((rot.1).0), 20.);
    assert_eq!(f32::round((rot.2).0), 30.);

    // Rotate local and global produce same, correct results for default
    let mut trans1 = Transform::default();
    let mut trans2 = Transform::default();

    let rot1 = trans1
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .rotation()
    ;

    let rot2 = trans2
        .rotate_local([1., 0., 0.].into(), Deg(50.))
        .rotation()
    ;

    assert_eq!(f32::round((rot1.0).0), 50.);
    assert_eq!(f32::round((rot1.1).0), 0.);
    assert_eq!(f32::round((rot1.2).0), 0.);
    assert_eq!(f32::round((rot2.0).0), 50.);
    assert_eq!(f32::round((rot2.1).0), 0.);
    assert_eq!(f32::round((rot2.2).0), 0.);
}

#[test]
fn multi_rotation() {
    // Rotate around multiple axes
    let mut trans = Transform::default();
    let rot = trans
        .set_rotation(Deg(10.), Deg(20.), Deg(30.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 10.);
    assert_eq!(f32::round((rot.1).0), 20.);
    assert_eq!(f32::round((rot.2).0), 30.);

    // Rotate additively
    let mut trans = Transform::default();
    let rot = trans
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 100.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);

    let mut trans = Transform::default();
    let rot = trans
        .rotate_local([1., 0., 0.].into(), Deg(50.))
        .rotate_local([1., 0., 0.].into(), Deg(50.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 100.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}

#[test]
fn scaling() {
    // Set and get scale
    let mut trans = Transform::default();
    let scale = trans
        .set_scale(10., 20., 30.)
        .scale()
    ;

    assert_eq!(scale.0, 10.);
    assert_eq!(scale.1, 20.);
    assert_eq!(scale.2, 30.);
}

#[test]
fn mov_rot() {
    let mut trans = Transform::default();
    let trans = trans
        .move_global([50., 0., 0.].into())
        .rotate_global([1., 0., 0.].into(), Deg(50.))
    ;

    let pos = trans.position();
    let rot = trans.rotation();
    let scale = trans.scale();

    assert_eq!(pos.x, 50.);
    assert_eq!(pos.y, 0.);
    assert_eq!(pos.z, 0.);

    assert_eq!(scale.0, 1.);
    assert_eq!(scale.1, 1.);
    assert_eq!(scale.2, 1.);

    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}

#[test]
fn scale_rot() {
    let mut trans = Transform::default();
    let trans = trans
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .set_scale(10., 20., 30.)
    ;

    let pos = trans.position();
    let rot = trans.rotation();
    let scale = trans.scale();

    assert_eq!(pos.x, 0.);
    assert_eq!(pos.y, 0.);
    assert_eq!(pos.z, 0.);

    assert_eq!(scale.0, 10.);
    assert_eq!(scale.1, 20.);
    assert_eq!(scale.2, 30.);

    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}

#[test]
fn full_transform() {
    let mut trans = Transform::default();
    let trans = trans
        .move_global([50., 0., 0.].into())
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .set_scale(10., 20., 30.)
    ;

    let pos = trans.position();
    let rot = trans.rotation();
    let scale = trans.scale();

    assert_eq!(pos.x, 50.);
    assert_eq!(pos.y, 0.);
    assert_eq!(pos.z, 0.);

    assert_eq!(scale.0, 10.);
    assert_eq!(scale.1, 20.);
    assert_eq!(scale.2, 30.);

    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}
