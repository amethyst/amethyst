//! Utility to adjust the aspect ratio of cameras automatically

use amethyst_assets::PrefabData;
use amethyst_core::ecs::{
    Component, Entity, HashMapStorage, Join, ReadExpect, ReadStorage, System, SystemData,
    WriteStorage,
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
    /// The desired vertical fov
    fov_y: f32,
    /// Distance to the near plane
    z_near: f32,
}

impl AutoFov {
    /// Creates a new instance with vertical fov of pi/3 and near plane of 0.125.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for AutoFov {
    fn default() -> Self {
        Self {
            fov_y: std::f32::consts::FRAC_PI_3,
            z_near: 0.125,
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
        ReadStorage<'a, AutoFov>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, (screen, auto_fovs, mut cameras): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("auto_fov_system");

        if self.last_dimensions != *screen {
            for (camera, auto_fov) in (&mut cameras, &auto_fovs).join() {
                *camera =
                    Camera::perspective(screen.aspect_ratio(), auto_fov.fov_y, auto_fov.z_near)
            }
            self.last_dimensions = screen.clone();
        }
    }
}

impl Default for AutoFovSystem {
    fn default() -> Self {
        Self {
            last_dimensions: ScreenDimensions::new(0, 0, 0.0),
        }
    }
}
