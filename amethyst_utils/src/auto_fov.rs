//! Utility to adjust the aspect ratio of cameras automatically

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::{
    Component, Entity, HashMapStorage, Join, ReadExpect, ReadStorage, System, WriteStorage,
};
use amethyst_derive::PrefabData;
use amethyst_renderer::{Camera, ScreenDimensions};

use serde::{Deserialize, Serialize};

/// A component describing the behavior of the camera in accordance with the screen dimensions
#[derive(Clone, Deserialize, PrefabData, Serialize)]
#[prefab(Component)]
#[serde(default)]
pub struct AutoFov {
    /// The horizontal FOV value at the aspect ratio in the field `base_aspect_ratio`
    base_fovx: f32,

    /// The factor determining how sensitive the FOV change should be
    fovx_growth_rate: f32,

    /// If the FOV grow rate specified in the field `fovx_growth_rate` should be applied as-is
    fixed_growth_rate: bool,

    /// The aspect ratio when the camera's horizontal FOV is identical to `base_fovx`
    base_aspect_ratio: (usize, usize),

    /// The minimum value the horizontal FOV can have
    min_fovx: f32,

    /// The maximum value the horizontal FOV can have
    max_fovx: f32,
}

impl AutoFov {
    /// Creates a new instance with the default values for all fields
    pub fn new() -> Self {
        Default::default()
    }

    /// The horizontal FOV value at the aspect ratio in the field `base_aspect_ratio`
    ///
    /// This value should be between the `min_fovx` and `max_fovx` values. Value in radians.
    /// Defaults to `1.861684535`, which is the horizontal FOV value when the vertial FOV and aspect
    /// ratio for a camera is set to `1.0471975512`(60 deg) and `16/9`, respectively.
    pub fn base_fovx(&self) -> f32 {
        self.base_fovx
    }

    /// The factor determining how sensitive the FOV change should be
    ///
    /// Defaults to `1.0`.
    pub fn fovx_growth_rate(&self) -> f32 {
        self.fovx_growth_rate
    }

    /// If the FOV grow rate specified in the field `fovx_growth_rate` should be applied as-is
    ///
    /// Defaults to `false`. When `false`, the `fovx_growth_rate` field is multiplied with the
    /// camera's current vertical FOV value.
    pub fn fixed_growth_rate(&self) -> bool {
        self.fixed_growth_rate
    }

    /// The aspect ratio when the camera's horizontal FOV is identical to `base_fovx`
    ///
    /// Defaults to `(16, 9)`.
    pub fn base_aspect_ratio(&self) -> (usize, usize) {
        self.base_aspect_ratio
    }

    /// The minimum value the horizontal FOV can have
    ///
    /// This value should be larger than 0. Defaults to `0.1`.
    pub fn min_fovx(&self) -> f32 {
        self.min_fovx
    }

    /// The maximum value the horizontal FOV can have
    ///
    /// This value should be larger than 0. Defaults to `PI`. The rendered view will be stretched if
    /// the screen aspect ratio keeps growing after the point where the camera's horizontal FOV
    /// reaches this maximum value.
    pub fn max_fovx(&self) -> f32 {
        self.max_fovx
    }

    /// Sets `base_fovx` to the given value
    ///
    /// This function panics if the given value is not between `min_fovx` and `max_fovx`.
    pub fn set_base_fovx(&mut self, base_fovx: f32) {
        assert!(
            base_fovx >= self.min_fovx,
            format!(
                "`base_fovx` should be larger than `min_fovx` which is `{}`, but `{}` given",
                self.min_fovx, base_fovx
            )
        );
        assert!(
            base_fovx <= self.max_fovx,
            format!(
                "`base_fovx` should be smaller than `max_fovx` which is `{}`, but `{}` given",
                self.max_fovx, base_fovx
            )
        );

        self.base_fovx = base_fovx;
    }

    /// Sets `fovx_growth_rate` to the given value
    pub fn set_fovx_growth_rate(&mut self, fovx_growth_rate: f32) {
        self.fovx_growth_rate = fovx_growth_rate;
    }

    /// Sets `fixed_growth_rate` to the given value
    ///
    /// You can optionally give the new `fovx_growth_rate` value to be set.
    pub fn set_fixed_growth_rate(&mut self, fix: bool, new_growth_rate: Option<f32>) {
        self.fixed_growth_rate = fix;

        if let Some(fovx_growth_rate) = new_growth_rate {
            self.fovx_growth_rate = fovx_growth_rate;
        }
    }

    /// Sets `base_aspect_ratio` to the given value
    ///
    /// This function panics if the horizontal or vertical ratio value is zero.
    pub fn set_base_aspect_ratio(&mut self, horizontal: usize, vertical: usize) {
        assert!(
            horizontal != 0,
            "The horizontal value of aspect ratio should be larger than 0"
        );
        assert!(
            vertical != 0,
            "The vertical value of aspect ratio should be larger than 0"
        );
        self.base_aspect_ratio = (horizontal, vertical);
    }

    /// Sets `min_fovx` to the given value
    ///
    /// This function panics if the given `min_fovx` is not larger than zero or is larger than
    /// `max_fovx`.
    pub fn set_min(&mut self, min: f32) {
        let max = self.max_fovx;
        self.set_min_max(min, max);
    }

    /// Sets `max_fovx` to the given value
    ///
    /// This function panics if the given `max_fovx` is smaller than `min_fovx`.
    pub fn set_max(&mut self, max: f32) {
        let min = self.min_fovx;
        self.set_min_max(min, max);
    }

    /// Sets `min_fovx` and `max_fovx` to the given vaues
    ///
    /// This function panics if the given `min_fovx` is not larger than zero, or if the given
    /// `max_fovx` is smaller than the given `min_fovx`.
    pub fn set_min_max(&mut self, min: f32, max: f32) {
        assert!(
            min > 0.0,
            format!("`min_fovx` should be larger than 0, but `{}` given", min)
        );
        assert!(
            max >= min,
            format!(
                "`max_fovx` should be larger than or equal to `min_fovx` which is `{}`, but `{}` given",
                min,
                max,
            ),
        );
        self.min_fovx = min;
        self.max_fovx = max;
    }

    /// Computes the new horizontal FOV from the current screen aspect ratio and vertical FOV
    pub fn new_fovx(&self, current_aspect_ratio: f32, fovy: f32) -> f32 {
        let delta_aspect = current_aspect_ratio - self.base_aspect_value();

        if delta_aspect.abs() <= ::std::f32::EPSILON {
            return self.base_fovx;
        }

        let fovy = if self.fixed_growth_rate { 1.0 } else { fovy };
        let delta_fovx = self.fovx_growth_rate * fovy * delta_aspect;
        let new_fovx = self.base_fovx + delta_fovx;

        new_fovx.max(self.min_fovx).min(self.max_fovx)
    }

    #[inline]
    fn base_aspect_value(&self) -> f32 {
        self.base_aspect_ratio.0 as f32 / self.base_aspect_ratio.1 as f32
    }
}

impl Component for AutoFov {
    type Storage = HashMapStorage<Self>;
}

impl Default for AutoFov {
    fn default() -> Self {
        AutoFov {
            base_fovx: 1.861684535,
            fovx_growth_rate: 1.0,
            fixed_growth_rate: false,
            base_aspect_ratio: (16, 9),
            min_fovx: 0.1,
            max_fovx: std::f32::consts::PI,
        }
    }
}

/// System that automatically adjusts the horizontal FOV based on the screen dimensions
///
/// For a camera component to be managed by this system, the entity with the camera component should
/// also have an `AutoFov` component attached to it.
///
/// If the camera is being loaded by a prefab, it is best to have the `PrefabLoaderSystem` loading
/// the camera as a dependency of this system. It enables the system to adjust the camera right
/// after it is created -- simply put, in the same frame.
pub struct AutoFovSystem;

impl<'a> System<'a> for AutoFovSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        ReadStorage<'a, AutoFov>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, (screen, auto_fovs, mut cameras): Self::SystemData) {
        let current_aspect = screen.aspect_ratio();

        for (camera, auto_fov) in (&mut cameras, &auto_fovs).join() {
            let fovy = get_fovy(camera);
            let fovx = auto_fov.new_fovx(current_aspect, fovy);
            set_aspect(camera, fovx / fovy);
        }
    }
}

#[inline]
fn get_fovy(camera: &Camera) -> f32 {
    (1.0 / camera.proj[(1, 1)]).atan() * 2.0
}

#[inline]
fn set_aspect(camera: &mut Camera, new_aspect: f32) {
    camera.proj[(0, 0)] = camera.proj[(1, 1)] / new_aspect;
}
