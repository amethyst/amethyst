//! Local transform component.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

use cgmath::{Euler, Matrix3, Matrix4, Quaternion, Vector3, Vector4};

use ecs::{Component, VecStorage};

/// Raw transform data.
#[derive(Debug)]
pub struct InnerTransform {
    /// Forward vector [x, y, z]
    pub forward: [f32; 3],
    /// Right vector [x, y, z]
    pub right: [f32; 3],
    /// Quaternion [w (scalar), x, y, z]
    pub rotation: [f32; 4],
    /// Scale vector [x, y, z]
    pub scale: [f32; 3],
    /// Translation/position vector [x, y, z]
    pub translation: [f32; 3],
    /// Up vector [x, y, z]
    pub up: [f32; 3],
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

    /// Rotate to look at a point in space (without rolling)
    pub fn look_at(&mut self, position: Vector3<f32>) -> &mut Self {
        let cam_quat = Quaternion::from(self.rotation);
        let pos_vec = Vector3::from(self.translation;

        self.rotation = cam_quat.look_at(position - pos_vec, self.up).into();
        self.flag(true);

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

    /// Move relatively to its current position and orientation.
    pub fn move_forward(&mut self, amount: f32) -> &mut Self {
        self.move_local(self.forward, amount)
    }

    /// Move relatively to its current position, but independently from its orientation.
    /// Ideally, first normalize the direction and then multiply it
    /// by whatever amount you want to move before passing the vector to this method
    #[inline]
    pub fn move_global(&mut self, direction: Vector3<f32>) -> &mut Self {
        self.translation = (Vector3::from(self.translation) + direction).into();
        self.flag(true);
        self
    }

    /// Move relatively to its current position and orientation.
    #[inline]
    pub fn move_local(&mut self, axis: Vector3<f32>, amount: f32) -> &mut Self {
        self.translation += Quaternion::from(self.rotation).conjugate() * axis.normalize() * amount;
        self
    }

    /// Move relatively to its current position and orientation.
    pub fn move_right(&mut self, amount: f32) -> &mut Self {
        self.move_local(self.right, amount)
    }

    /// Move relatively to its current position and orientation.
    pub fn move_up(&mut self, amount: f32) -> &mut Self {
        self.move_local(self.up, amount)
    }

    /// Pitch relatively to the world.
    pub fn pitch_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(self.right, angle)
    }

    /// Pitch relatively to its own rotation.
    pub fn pitch_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(self.right, angle)
    }

    /// Roll relatively to the world.
    pub fn roll_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(self.forward, angle)
    }

    /// Roll relatively to its own rotation.
    pub fn roll_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(self.forward, angle);
    }

    /// Add a rotation to the current rotation
    #[inline]
    pub fn rotate(&mut self, quat: Quaternion<f32>) -> &mut Self {
        self.rotation = quat * Quaternion::from(self.rotation);
        self.flag(true);
        self
    }

    /// Rotate relatively to the world
    #[inline]
    pub fn rotate_global(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let axis_normalized = Vectro3::from(axis).normalize();
        let q = Quaternion::from::<f32>(Euler {
            x: axis_normalized.x * angle,
            y: axis_normalized.y * angle,
            z: axis_normalized.z * angle,
        });

        self.rotate(q)
    }

    /// Rotate relatively to the current orientation
    #[inline]
    pub fn rotate_local(&mut self, axis: Vector3<f32>, angle: Deg<f32>) -> &mut Self {
        let rel_axis_normalized = Quaternion::from(self.rotation)
            .rotate_vector(Vector3::from(axis))
            .normalize();
        let q = Quaternion::from::<f32>(Euler {
            x: rel_axis_normalized.x * angle,
            y: rel_axis_normalized.y * angle,
            z: rel_axis_normalized.z * angle,
        });

        self.rotate(q)
    }

    /// Set the position.
    pub fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.translation = position.into();
        self.flag(true);

        self
    }

    /// Set the rotation using Euler x, y, z.
    pub fn set_rotation<D: Into<Deg<f32>>>(&mut self, x: D, y: D, z: D) -> &mut Self {
        self.rotation = Quaternion::from::<f32>(Euler {
            x: x,
            y: y,
            z: z,
        }).into();

        self.flag(true);
        self
    }

    /// Yaw relatively to the world.
    pub fn yaw_global(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_global(self.up, angle)
    }

    /// Yaw relatively to its own rotation.
    pub fn yaw_local(&mut self, angle: Deg<f32>) -> &mut Self {
        self.rotate_local(self.up, angle)
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        LocalTransform {
            wrapped: InnerTransform {
                up:          [0.0, 0.0, 1.0],
                forward:     [1.0, 0.0, 0.0],
                right:       [0.0,-1.0, 0.0],

                rotation:    [1.0, 0.0, 0.0, 0.0],
                scale:       [1.0, 1.0, 1.0],
                translation: [0.0, 0.0, 0.0],
            },
            dirty: AtomicBool::new(true),
        }
    }
}

impl Component for LocalTransform {
    type Storage = VecStorage<LocalTransform>;
}
