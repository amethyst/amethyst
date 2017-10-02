//! Local transform component.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};

use ecs::{Component, VecStorage};

/// Raw transform data.
#[derive(Debug)]
pub struct InnerTransform {
    /// Translation/position vector [x, y, z]
    pub translation: [f32; 3],
    /// Quaternion [w (scalar), x, y, z]
    pub rotation: [f32; 4],
    /// Scale vector [x, y, z]
    pub scale: [f32; 3],
}

/// Local position, rotation, and scale (from parent if it exists).
///
/// Used for rendering position and orientation.
#[derive(Debug)]
pub struct LocalTransform {
    /// Wrapper around the transform data for dirty flag setting.
    wrapped: InnerTransform,
    /// Flag for re-computation
    dirty: AtomicBool,
}

impl Deref for LocalTransform {
    type Target = InnerTransform;
    fn deref(&self) -> &InnerTransform {
        &self.wrapped
    }
}

impl DerefMut for LocalTransform {
    fn deref_mut(&mut self) -> &mut InnerTransform {
        self.flag(true);
        &mut self.wrapped
    }
}

impl LocalTransform {
    /// Flags the current transform for re-computation.
    ///
    /// Note: All `set_*` methods will automatically flag the component.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether or not the current transform is flagged for
    /// re-computation or "dirty".
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Returns the forward-vector of the transform
    /// Useful for camera calculations
    /// For the camera, the center-point can be calculated as position+forward
    pub fn forward(&self) -> [f32; 3] {

    }

    /// Rotate to look at a point in space (without rolling)
    pub fn look_at(&mut self, position: Vector3<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's global `Transform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let quat: Matrix3<f32> = Quaternion::from(self.rotation).into();
        let scale: Matrix3<f32> = Matrix3::<f32> {
            x: [self.scale[0], 0.0, 0.0].into(),
            y: [0.0, self.scale[1], 0.0].into(),
            z: [0.0, 0.0, self.scale[2]].into(),
        };
        let mut matrix: Matrix4<f32> = (&quat * scale).into();
        matrix.w = Vector3::from(self.translation).extend(1.0f32);
        matrix.into()
    }

    /// Move the camera relatively to its current position, but independently from its orientation.
    pub fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self {
        self.dirty = true;
        self.eye += direction;
        self
    }

    /// Move the camera relatively to its current position and orientation.
    pub fn move_local(&mut self, direction: Vector3<f32>) -> &mut Self {
        self.dirty = true;
        self.eye += self.rotation * direction;
        self
    }

    /// Pitch the camera relatively to the world.
    pub fn pitch_global(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Pitch the camera relatively to its own rotation.
    pub fn pitch_local(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Roll the camera relatively to the world.
    pub fn roll_global(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Roll the camera relatively to its own rotation.
    pub fn roll_local(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Set the position of the camera.
    pub fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.dirty = true;
        self.eye = position;
        self
    }

    /// Set the rotation of the camera using Euler x, y, z.
    pub fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self {
        self.dirty = true;
        //self.rotation = Quaternion::from::<f32>(Euler {
        //    x: x,
        //    y: y,
        //    z: z,
        //});

        self
    }

    /// Returns the up-vector of the transform
    /// Useful for camera calculations
    pub fn up(&self) -> [f32; 3] {

    }

    /// Yaw the camera relatively to the world.
    pub fn yaw_global(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }

    /// Yaw the camera relatively to its own rotation.
    pub fn yaw_local(&mut self, angle: Deg<f32>) -> &mut Self {
        // tbd
        self
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        LocalTransform {
            wrapped: InnerTransform {
                translation: [0.0, 0.0, 0.0],
                rotation: [1.0, 0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            dirty: AtomicBool::new(true),
        }
    }
}

impl Component for LocalTransform {
    type Storage = VecStorage<LocalTransform>;
}
