//! Utility to adjust the aspect ratio of cameras automatically

use amethyst_core::ecs::*;
use amethyst_rendy::camera::Camera;
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// A component that stores the parameters that the associated camera should have
/// when it is managed by the AutoFovSystem.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// System that automatically adjusts the horizontal FOV based on the screen dimensions
///
/// For a camera component to be managed by this system, the entity with the camera component should
/// also have an `AutoFov` component attached to it.
#[derive(Debug)]
pub struct AutoFovSystem;

impl System for AutoFovSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        let mut last_dimensions = ScreenDimensions::new(0, 0);

        Box::new(
            SystemBuilder::new("auto_fov_system")
                .read_resource::<ScreenDimensions>()
                .with_query(<(Write<Camera>, Write<AutoFov>)>::query())
                .build(move |_commands, subworld, screen, query| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("auto_fov_system");

                    for (camera, auto_fov) in query.iter_mut(subworld) {
                        if last_dimensions != **screen || auto_fov.dirty {
                            *camera = Camera::perspective(
                                screen.aspect_ratio(),
                                auto_fov.fov_y,
                                auto_fov.z_near,
                            );
                            auto_fov.dirty = false;
                        }
                    }
                    last_dimensions = screen.clone();
                }),
        )
    }
}
