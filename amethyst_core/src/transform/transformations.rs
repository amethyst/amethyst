//! Movement Traits

use std;

use cgmath::{Angle, Deg, Point3, Rad, Vector3};
use orientation::Orientation;

pub trait Move {
    /// Move relatively to its current position and orientation.
    fn move_backward(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;

    /// Move relatively to its current position and orientation.
    fn move_down(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;

    /// Move relatively to its current position and orientation.
    fn move_forward(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;
    
    /// Move relatively to its current position, but independently from its orientation.
    /// Ideally, first normalize the direction and then multiply it
    /// by whatever amount you want to move before passing the vector to this method
    fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self;
    
    /// Move relatively to its current position and orientation.
    fn move_left(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;
    
    /// Move relatively to its current position and orientation.
    fn move_local(&mut self, axis: Vector3<f32>, amount: f32) -> &mut Self;
    
    /// Move relatively to its current position and orientation.
    fn move_right(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;
    
    /// Move relatively to its current position and orientation.
    fn move_up(&mut self, orientation: &Orientation, amount: f32) -> &mut Self;
    
    /// Get current position
    fn position(&self) -> Point3<f32>;
    
    /// Set the position.
    fn set_position(&mut self, position: Point3<f32>) -> &mut Self;
}

pub trait Pitch {
    /// Pitch relatively to the world.
    fn pitch_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
    
    /// Pitch relatively to its own rotation.
    fn pitch_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
}

pub trait Roll {
    /// Roll relatively to the world.
    fn roll_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
    
    /// Roll relatively to its own rotation.
    fn roll_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
}

pub trait Rotate {
    /// Rotate to look at a point in space (without rolling)
    fn look_at(&mut self, orientation: &Orientation, position: Point3<f32>) -> &mut Self;
    
    /// Rotate whole local translation around a point at center via an axis and an angle
    fn rotate_around(&mut self, center: Point3<f32>, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self;
    
    /// Rotate relatively to the world
    fn rotate_global(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self;
    
    /// Rotate relatively to the current orientation
    fn rotate_local(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self;
    
    /// Get current rotation as x, y, z degree values
    fn rotation(&self) -> (Deg<f32>, Deg<f32>, Deg<f32>);
    
    /// Set the rotation using Euler x, y, z.
    fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self where D: Angle, Rad<<D as Angle>::Unitless>: std::convert::From<D>;
}

pub trait Scale {
    /// Get current scale as x, y, z values
    fn scale(&self) -> (f32, f32, f32);

    // Set new scale
    fn set_scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self;
}

pub trait Yaw {
    /// Yaw relatively to the world.
    fn yaw_global(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
    
    /// Yaw relatively to its own rotation.
    fn yaw_local(&mut self, orientation: &Orientation, angle: Deg<f32>) -> &mut Self;
}
