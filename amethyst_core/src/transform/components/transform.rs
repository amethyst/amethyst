//! Global transform component.

use std::borrow::Borrow;

use cgmath::{Deg, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4, One, Point3, Quaternion,
             Rotation, Vector3};
use orientation::Orientation;
use specs::{Component, DenseVecStorage, FlaggedStorage};
use super::super::{Move, Pitch, Roll, Rotate, Scale, Yaw};

/// Performs a global transformation on the entity (transform from origin).
///
/// Used for rendering position and orientation.
///
/// If this component is used, and `TransformSystem` is not used, then make sure to clear the flags
/// on the `FlaggedStorage` at the appropriate times (before updating any `Transform` in the frame).
/// See documentation on `FlaggedStorage` for more information.
#[derive(Debug, Copy, Clone)]
pub struct GlobalTransform(pub Matrix4<f32>);

impl GlobalTransform {
    /// Checks whether each `f32` of the `GlobalTransform` is finite (not NaN or inf).
    pub fn is_finite(&self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if !self.0[i][j].is_finite() {
                    return false;
                }
            }
        }

        true
    }
}

impl Component for GlobalTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Default for GlobalTransform {
    fn default() -> Self {
        GlobalTransform(Matrix4::one())
    }
}

impl GlobalTransform {
    /// Creates a new `GlobalTransform` in the form of an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<[[f32; 4]; 4]> for GlobalTransform {
    fn from(matrix: [[f32; 4]; 4]) -> Self {
        GlobalTransform(matrix.into())
    }
}

impl Into<[[f32; 4]; 4]> for GlobalTransform {
    fn into(self) -> [[f32; 4]; 4] {
        self.0.into()
    }
}

impl AsRef<[[f32; 4]; 4]> for GlobalTransform {
    fn as_ref(&self) -> &[[f32; 4]; 4] {
        self.0.as_ref()
    }
}

impl Borrow<[[f32; 4]; 4]> for GlobalTransform {
    fn borrow(&self) -> &[[f32; 4]; 4] {
        self.0.as_ref()
    }
}

impl Move for GlobalTransform {
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
    fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self {
        //self.0 = Matrix4::from_translation(direction) * self.0;
        let new_pos = self.position().to_vec() + direction;
        let rot = self.rotation();
        let scale = self.scale();

        self.0 =
            Matrix4::from_translation(new_pos) *
                Matrix4::from_angle_x::<Deg<f32>>(rot.0) *
                Matrix4::from_angle_y::<Deg<f32>>(rot.1) *
                Matrix4::from_angle_z::<Deg<f32>>(rot.2) *
                Matrix4::from_nonuniform_scale(scale.0, scale.1, scale.2)
        ;

        self
    }

    /// Move relatively to its current position and orientation.
    fn move_left(&mut self, orientation: &Orientation, amount: f32) -> &mut Self {
        self.move_local(orientation.right.into(), -amount)
    }

    /// Move relatively to its current position and orientation.
    fn move_local(&mut self, axis: Vector3<f32>, amount: f32) -> &mut Self {
        let scale = self.scale();
        let q = Quaternion::from(Matrix3::new(
            self.0[0][0] / scale.0,
            self.0[0][1] / scale.0,
            self.0[0][2] / scale.0,

            self.0[1][0] / scale.1,
            self.0[1][1] / scale.1,
            self.0[1][2] / scale.1,

            self.0[2][0] / scale.2,
            self.0[2][1] / scale.2,
            self.0[2][2] / scale.2,
        ));
        let delta = q.conjugate().invert() * axis.normalize() * amount;

        self.move_global(delta)
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
        Point3::new(
            self.0[3][0],
            self.0[3][1],
            self.0[3][2],
        )
    }

    /// Set the position.
    fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        let pos = self.position();

        self.move_global(Vector3::new(
            position[0] - pos[0],
            position[1] - pos[1],
            position[2] - pos[2],
        ))
    }
}

impl Pitch for GlobalTransform {
    /// Pitch relatively to the world.
    fn pitch_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.right.into(), angle)
    }

    /// Pitch relatively to its own rotation.
    fn pitch_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.right.into(), angle)
    }
}

impl Roll for GlobalTransform {
    /// Roll relatively to the world.
    fn roll_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(orientation.forward.into(), angle)
    }

    /// Roll relatively to its own rotation.
    fn roll_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(orientation.forward.into(), angle)
    }
}

impl Rotate for GlobalTransform {
    /// Rotate to look at a point in space (without rolling)
    fn look_at(&mut self, orientation: &Orientation, position: Point3<f32>) -> &mut Self {
        self.0 = Matrix4::look_at(self.position(), position, Vector3::from(orientation.up));
        self
    }

    /// Rotate whole local translation around a point at center via an axis and an angle
    fn rotate_around(&mut self, center: Point3<f32>, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        self.move_global(-center.to_vec());
        self.rotate_global(axis, angle);
        self.move_global(center.to_vec());
        self
    }

    /// Rotate relatively to the world
    #[inline]
    fn rotate_global(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let axis_normalized = Vector3::from(axis).normalize();

        self.0 = Matrix4::from_axis_angle(axis_normalized, angle) * self.0;
        self
    }

    /// Rotate relatively to the current orientation
    #[inline]
    fn rotate_local(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let axis_normalized = Vector3::from(axis).normalize();

        self.0 = self.0 * Matrix4::from_axis_angle(axis_normalized, angle);
        self
    }

    /// Get current rotation as x, y, z degree values
    fn rotation(&self) -> (Deg<f32>, Deg<f32>, Deg<f32>) {
        let c0 = Vector3::new(
            self.0[0][0],
            self.0[1][0],
            self.0[2][0],
        ).normalize();

        let c1 = Vector3::new(
            self.0[0][1],
            self.0[1][1],
            self.0[2][1],
        ).normalize();

        let c2 = Vector3::new(
            self.0[0][2],
            self.0[1][2],
            self.0[2][2],
        ).normalize();

        let angles = Euler::from(Quaternion::from(Matrix3::new(
            c0.x, c1.x, c2.x,
            c0.y, c1.y, c2.y,
            c0.z, c1.z, c2.z,
        )));

        (
            angles.x.into(),
            angles.y.into(),
            angles.z.into(),
        )
    }

    /// Set the rotation using Euler x, y, z.
    fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self {
        let scale = self.scale();

        self.0 =
            Matrix4::from_translation(self.position().to_vec()) *
                Matrix4::from_angle_x::<Deg<f32>>(x.into()) *
                Matrix4::from_angle_y::<Deg<f32>>(y.into()) *
                Matrix4::from_angle_z::<Deg<f32>>(z.into()) *
                Matrix4::from_nonuniform_scale(scale.0, scale.1, scale.2)
        ;

        self
    }
}

impl Scale for GlobalTransform {
    /// Get current scale as x, y, z values
    fn scale(&self) -> (f32, f32, f32) {
        (
            Vector3::new(self.0[0][0], self.0[0][1], self.0[0][2]).magnitude(),
            Vector3::new(self.0[1][0], self.0[1][1], self.0[1][2]).magnitude(),
            Vector3::new(self.0[2][0], self.0[2][1], self.0[2][2]).magnitude(),
        )
    }

    // Set new scale
    fn set_scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let rot = self.rotation();

        self.0 =
            Matrix4::from_translation(self.position().to_vec()) *
                Matrix4::from_angle_x::<Deg<f32>>(rot.0) *
                Matrix4::from_angle_y::<Deg<f32>>(rot.1) *
                Matrix4::from_angle_z::<Deg<f32>>(rot.2) *
                Matrix4::from_nonuniform_scale(x, y, z)
        ;

        self
    }
}

impl Yaw for GlobalTransform {
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
    let mut trans = GlobalTransform::default();
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
    let mut trans1 = GlobalTransform::default();
    let mut trans2 = GlobalTransform::default();

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
    let mut trans = GlobalTransform::default();
    let rot = trans
        .set_rotation(Deg(50.), Deg(0.), Deg(0.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);

    // Rotate local and global produce same, correct results for default
    let mut trans1 = GlobalTransform::default();
    let mut trans2 = GlobalTransform::default();

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
    let mut trans = GlobalTransform::default();
    let rot = trans
        .set_rotation(Deg(10.), Deg(20.), Deg(30.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 10.);
    assert_eq!(f32::round((rot.1).0), 20.);
    assert_eq!(f32::round((rot.2).0), 30.);

    // Rotate additively
    let mut trans = GlobalTransform::default();
    let rot = trans
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .rotation()
    ;

    assert_eq!(f32::round((rot.0).0), 100.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);

    let mut trans = GlobalTransform::default();
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
    let mut trans = GlobalTransform::default();
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
    let mut trans = GlobalTransform::default();
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
    let mut trans = GlobalTransform::default();
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

    println!("SR3, is: {:?}", rot);
    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}

#[test]
fn full_transform() {
    let mut trans = GlobalTransform::default();
    let trans = trans
        .rotate_global([1., 0., 0.].into(), Deg(50.))
        .set_scale(10., 20., 30.)
        .move_global([50., 100., 50.].into())
    ;

    let pos = trans.position();
    let rot = trans.rotation();
    let scale = trans.scale();

    println!("FT1, is: {:?}", pos);
    assert_eq!(pos.x, 50.);
    assert_eq!(pos.y, 100.);
    assert_eq!(pos.z, 50.);

    assert_eq!(scale.0, 10.);
    assert_eq!(scale.1, 20.);
    assert_eq!(scale.2, 30.);

    println!("{:?}", rot);
    assert_eq!(f32::round((rot.0).0), 50.);
    assert_eq!(f32::round((rot.1).0), 0.);
    assert_eq!(f32::round((rot.2).0), 0.);
}
