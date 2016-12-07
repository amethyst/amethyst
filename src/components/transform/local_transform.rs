extern crate cgmath;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};
use ecs::{Component, VecStorage};
use std::sync::atomic::{AtomicBool, Ordering};

/// Local position, rotation, and scale (from parent if it exists).
#[derive(Debug)]
pub struct LocalTransform {
    /// Translation/position vector [x, y, z]
    translation: [f32; 3],

    /// Quaternion [w (scalar), x, y, z]
    rotation: [f32; 4],

    /// Scale vector [x, y, z]
    scale: [f32; 3],

    /// Flag for re-computation
    dirty: AtomicBool,
}

impl LocalTransform {
    #[inline]
    pub fn translation(&self) -> [f32; 3] {
        self.translation
    }
    #[inline]
    pub fn rotation(&self) -> [f32; 4] {
        self.rotation
    }
    #[inline]
    pub fn scale(&self) -> [f32; 3] {
        self.scale
    }
    #[inline]
    pub fn set_translation(&mut self, translation: [f32; 3]) {
        self.translation = translation;
        self.flag(true);
    }
    #[inline]
    pub fn set_rotation(&mut self, rotation: [f32; 4]) {
        self.rotation = rotation;
        self.flag(true);
    }
    #[inline]
    pub fn set_scale(&mut self, scale: [f32; 3]) {
        self.scale = scale;
        self.flag(true);
    }

    /// Set a specific part of the translation/position without modifying the others
    /// (must be an index less than 3).
    ///
    /// Format: [0 = x, 1 = y, 2 = z]
    ///
    /// e.g. `transform.set_translation_index(1, 5.0)` sets `y` to `5.0`
    #[inline]
    pub fn set_translation_index(&mut self, index: usize, val: f32) {
        assert!(index < 3,
                "Attempted to use `set_pos_index` with an index higher than 2");
        self.translation[index] = val;
        self.flag(true);
    }

    /// Set a specific part of the rotation quaternion without modifying the others
    /// (must be an index less than 4).
    ///
    /// Format: [0 = w, 1 = x, 2 = y, 3 = z]
    ///
    /// e.g. `transform.set_rot_index(1, 0.0)` sets `x` to `0.0`
    #[inline]
    pub fn set_rotation_index(&mut self, index: usize, val: f32) {
        assert!(index < 4,
                "Attempted to use `set_rot_index` with an index higher than 3");
        self.rotation[index] = val;
        self.flag(true);
    }

    /// Set a specific part of the scale without modifying the others
    /// (must be an index less than 3).
    ///
    /// Format: [0 = x, 1 = y, 2 = z]
    ///
    /// e.g. `transform.set_scale_index(2, 3.0)` sets `z` to `3.0`
    #[inline]
    pub fn set_scale_index(&mut self, index: usize, val: f32) {
        assert!(index < 3,
                "Attempted to use `set_scale_index` with an index higher than 2");
        self.scale[index] = val;
        self.flag(true);
    }

    /// Flags the current transform for re-computation.
    ///
    /// Note: All `set_*` methods will automatically flag the component.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether or not the current transform is flagged for re-computation or "dirty".
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
            translation: [0.0, 0.0, 0.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            dirty: AtomicBool::new(true),
        }
    }
}

impl Component for LocalTransform {
    type Storage = VecStorage<LocalTransform>;
}
