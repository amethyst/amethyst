//! Local transform component.

use cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Component, VecStorage};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

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
