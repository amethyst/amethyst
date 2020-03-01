//! Utility to adjust the aspect ratio of cameras automatically

use amethyst_assets::PrefabData;
use amethyst_core::ecs::{
    Component, Entity, HashMapStorage, Join, ReadExpect, System, SystemData, WriteStorage,
};
use amethyst_derive::{PrefabData, SystemDesc};
use amethyst_error::Error;
use amethyst_rendy::camera::Camera;
use amethyst_window::ScreenDimensions;

use serde::{Deserialize, Serialize};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// A component that stores the parameters that the associated camera should have
/// when it is managed by the AutoFovSystem.
#[derive(Clone, Debug, Deserialize, PrefabData, Serialize)]
#[prefab(Component)]
#[serde(default)]
pub struct AutoFov {
    fov_y: f32,
    z_near: f32,
    // FOV has to be adjusted when the camera parameters change or when a new
    // camera is created.
    dirty: bool,
}

impl AutoFov {
    /// Creates a new instance with vertical fov of pi/3 and near plane of 0.125.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the vertical fov
    pub fn set_fov(&mut self, fov: f32) {
        self.fov_y = fov;
        self.dirty = true;
    }

    /// Set the distance to the near plane
    pub fn set_near(&mut self, near: f32) {
        self.z_near = near;
        self.dirty = true;
    }
}

impl Default for AutoFov {
    fn default() -> Self {
        Self {
            fov_y: std::f32::consts::FRAC_PI_3,
            z_near: 0.125,
            dirty: true,
        }
    }
}

impl Component for AutoFov {
    type Storage = HashMapStorage<Self>;
}

/// System that automatically adjusts the horizontal FOV based on the screen dimensions
///
/// For a camera component to be managed by this system, the entity with the camera component should
/// also have an `AutoFov` component attached to it.
///
/// If the camera is being loaded by a prefab, it is best to have the `PrefabLoaderSystem` loading
/// the camera as a dependency of this system. It enables the system to adjust the camera right
/// after it is created -- simply put, in the same frame.
#[derive(Debug, SystemDesc)]
pub struct AutoFovSystem {
    last_dimensions: ScreenDimensions,
}

impl AutoFovSystem {
    /// Sets up `SystemData` and returns a new `AutoFovSystem`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for AutoFovSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        WriteStorage<'a, AutoFov>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, (screen, mut auto_fovs, mut cameras): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("auto_fov_system");

        for (camera, auto_fov) in (&mut cameras, &mut auto_fovs).join() {
            if self.last_dimensions != *screen || auto_fov.dirty {
                *camera =
                    Camera::perspective(screen.aspect_ratio(), auto_fov.fov_y, auto_fov.z_near);
                auto_fov.dirty = false;
            }
        }
        self.last_dimensions = screen.clone();
    }
}

impl Default for AutoFovSystem {
    fn default() -> Self {
        Self {
            last_dimensions: ScreenDimensions::new(0, 0, 0.0),
        }
    }
}
