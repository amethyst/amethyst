//! Provides a automatically resized orthographic camera.

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::{Component, DenseVecStorage, Entity, Join, ReadExpect, System, WriteStorage},
    Axis2,
};
use amethyst_derive::PrefabData;
use amethyst_error::Error;
use amethyst_rendy::camera::{Camera, Orthographic};
use amethyst_window::ScreenDimensions;
use derive_new::new;

use serde::{Deserialize, Serialize};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// The coordinates that `CameraOrtho` will keep visible in the window.
/// `bottom` can be a higher value than `top`, as is common in 2D coordinates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct CameraOrthoWorldCoordinates {
    /// Left x coordinate
    pub left: f32,
    /// Right x coordinate
    pub right: f32,
    /// Bottom y coordinate
    pub bottom: f32,
    /// Top y coordinate
    pub top: f32,
}

impl CameraOrthoWorldCoordinates {
    /// Creates coordinates with (0,0) at the bottom left, and (1,1) at the top right
    pub fn normalized() -> CameraOrthoWorldCoordinates {
        CameraOrthoWorldCoordinates {
            left: 0.0,
            right: 1.0,
            bottom: 0.0,
            top: 1.0,
        }
    }

    /// Returns width / height of the desired camera coordinates.
    pub fn aspect_ratio(&self) -> f32 {
        self.width() / self.height()
    }

    /// Returns size of the x-axis.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Returns size of the y-axis.
    pub fn height(&self) -> f32 {
        // abs is in case you're using upside-down coordinates
        (self.top - self.bottom).abs()
    }
}

impl Default for CameraOrthoWorldCoordinates {
    fn default() -> Self {
        Self::normalized()
    }
}

/// `Component` attached to the camera's entity that allows automatically adjusting the camera's matrix according
/// to preferences in the "mode" and "world_coordinates" fields.
/// It adjusts the camera so that the camera's world coordinates are always visible.
/// You must add the `CameraOrthoSystem` to your dispatcher for this to take effect (no dependencies required).
///
/// # Example
///
/// ```rust
/// # use amethyst_core::ecs::{Builder, World, WorldExt};
/// # use amethyst_core::Transform;
/// # use amethyst_rendy::camera::Camera;
/// # use amethyst_utils::ortho_camera::*;
/// # let mut world = World::new();
/// # world.register::<Transform>();
/// # world.register::<Camera>();
/// # world.register::<CameraOrtho>();
/// world
///     .create_entity()
///     .with(Transform::default())
///     .with(Camera::standard_2d(1920.0, 1080.0))
///     .with(CameraOrtho::normalized(CameraNormalizeMode::Contain))
///     .build();
/// ```
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, PrefabData, new)]
#[prefab(Component)]
pub struct CameraOrtho {
    /// How the camera's matrix is changed when the window's aspect ratio changes.
    /// See `CameraNormalizeMode` for more info.
    pub mode: CameraNormalizeMode,
    /// The world coordinates that this camera will keep visible as the window size changes
    pub world_coordinates: CameraOrthoWorldCoordinates,
    #[new(default)]
    aspect_ratio_cache: f32,
}

impl CameraOrtho {
    /// Creates a Camera that maintains window coordinates of (0,0) in the bottom left, and (1,1) at the top right
    pub fn normalized(mode: CameraNormalizeMode) -> CameraOrtho {
        CameraOrtho {
            mode,
            world_coordinates: Default::default(),
            aspect_ratio_cache: 0.0,
        }
    }

    /// Get the camera matrix offsets according to the specified options.
    pub fn camera_offsets(&self, window_aspect_ratio: f32) -> (f32, f32, f32, f32) {
        self.mode
            .camera_offsets(window_aspect_ratio, &self.world_coordinates)
    }
}

impl Component for CameraOrtho {
    type Storage = DenseVecStorage<Self>;
}

/// Settings that decide how to scale the camera's matrix when the aspect ratio changes.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum CameraNormalizeMode {
    /// Using the aspect ratio from the world coordinates for this camera, tries to adjust the matrix values of the
    /// camera so that the orthogonal direction to the stretch_direction always have a world size of 1.
    ///
    /// This means that the direction opposite to stretch_direction
    /// will always be between 0.0 to 1.0 in world coordinates.
    /// Scene space can be lost (or gained) on the specified stretch_direction however.
    ///
    /// Example (using a normalized ortho camera):
    /// If you use Lossy with the stretch_direction of Axis::X,
    /// this means that a mesh or image going from the world coordinates (0, 0) to (1, 1)
    /// would take the whole screen size if the window dimension width is equal to its height.
    ///
    /// If the window gets stretched on the x axis, the mesh or image will stay centered and
    /// the background clear color (or things that were previously outside of the window) will now
    /// be shown on the left and right sides of the mesh or image.
    ///
    /// If you shrink the window on the x axis instead, the left and right parts of the images will go
    /// off screen and will NOT be shown.
    ///
    /// If you want the whole world space between (0, 0) and (1, 1) to be shown at ALL times, consider using
    /// `CameraNormalizeMode::Contain` instead.
    Lossy {
        /// The direction along which the camera will stretch and possibly have a length not equal
        /// to one.
        stretch_direction: Axis2,
    },

    /// Scales the render dynamically to ensure the `CameraOrthoWorldCoordinates` are always visible.
    /// There may still be additional space in addition to the specific coordinates, but it will never hide anything.
    ///
    /// If you have a non-default `Transform` on your camera,
    /// it will just translate those coordinates by the translation of the `Transform`.
    Contain,
}

impl CameraNormalizeMode {
    /// Get the camera matrix offsets according to the specified options.
    fn camera_offsets(
        self,
        window_aspect_ratio: f32,
        desired_coordinates: &CameraOrthoWorldCoordinates,
    ) -> (f32, f32, f32, f32) {
        match self {
            CameraNormalizeMode::Lossy {
                ref stretch_direction,
            } => match stretch_direction {
                Axis2::X => CameraNormalizeMode::lossy_x(window_aspect_ratio, desired_coordinates),
                Axis2::Y => CameraNormalizeMode::lossy_y(window_aspect_ratio, desired_coordinates),
            },
            CameraNormalizeMode::Contain => {
                let desired_aspect_ratio = desired_coordinates.aspect_ratio();
                // We don't need an == case because lossy handles it just fine
                if window_aspect_ratio > desired_aspect_ratio {
                    // The window is wide, bars should be on X
                    CameraNormalizeMode::lossy_x(window_aspect_ratio, desired_coordinates)
                } else {
                    CameraNormalizeMode::lossy_y(window_aspect_ratio, desired_coordinates)
                }
            }
        }
    }

    fn lossy_x(
        window_aspect_ratio: f32,
        desired_coordinates: &CameraOrthoWorldCoordinates,
    ) -> (f32, f32, f32, f32) {
        let offset = (window_aspect_ratio * desired_coordinates.height()
            - desired_coordinates.width())
            / 2.0;
        (
            desired_coordinates.left - offset,
            desired_coordinates.right + offset,
            desired_coordinates.bottom,
            desired_coordinates.top,
        )
    }

    fn lossy_y(
        window_aspect_ratio: f32,
        desired_coordinates: &CameraOrthoWorldCoordinates,
    ) -> (f32, f32, f32, f32) {
        // If bottom is higher than top (common in 2D graphics), we flip the offset
        let sign = if desired_coordinates.bottom > desired_coordinates.top {
            -1.0
        } else {
            1.0
        };
        let offset = (desired_coordinates.width() / window_aspect_ratio
            - desired_coordinates.height())
            / 2.0
            * sign;
        (
            desired_coordinates.left,
            desired_coordinates.right,
            desired_coordinates.bottom - offset,
            desired_coordinates.top + offset,
        )
    }
}

impl Default for CameraNormalizeMode {
    fn default() -> Self {
        CameraNormalizeMode::Contain
    }
}

/// System that automatically changes the camera matrix according to the settings in
/// the `CameraOrtho` attached to the camera entity.
#[derive(Default, Debug)]
pub struct CameraOrthoSystem;

impl<'a> System<'a> for CameraOrthoSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        WriteStorage<'a, Camera>,
        WriteStorage<'a, CameraOrtho>,
    );

    #[allow(clippy::float_cmp)] // cmp just used to recognize change
    fn run(&mut self, (dimensions, mut cameras, mut ortho_cameras): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("camera_ortho_system");

        let aspect = dimensions.aspect_ratio();

        for (camera, mut ortho_camera) in (&mut cameras, &mut ortho_cameras).join() {
            if aspect != ortho_camera.aspect_ratio_cache {
                ortho_camera.aspect_ratio_cache = aspect;
                let offsets = ortho_camera.camera_offsets(aspect);

                let (near, far) = if let Some(prev) = camera.projection().as_orthographic() {
                    (prev.near(), prev.far())
                } else {
                    continue;
                };

                camera.set_projection(
                    Orthographic::new(offsets.0, offsets.1, offsets.2, offsets.3, near, far).into(),
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ortho_camera::{CameraNormalizeMode, CameraOrtho, CameraOrthoWorldCoordinates};

    use super::Axis2;

    // TODO: Disabled until someone fixes the formula (if possible).
    /*#[test]
    fn near_far_from_camera() {
        use amethyst_core::cgmath::{Ortho, Matrix4};
        let mat4 = Matrix4::from(Ortho {
            left: 0.0,
            right: 1.0,
            bottom: 0.0,
            top: 1.0,
            near: 0.1,
            far: 2000.0,
        });
        let x = mat4.z.z; // c2r2
        let y = mat4.w.z; // c3r2
        let near = (y + 1.0) / x;
        let far = (x - 1.0) / y;
        assert_ulps_eq!((near as f32 * 100.0).round() / 100.0, 0.1);
        assert_ulps_eq!((far as f32 * 100.0).round() / 100.0, 2000.0);
    }*/

    #[test]
    fn normal_camera_large_lossy_horizontal() {
        let aspect = 2.0 / 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::X,
        });
        assert_eq!((-0.5, 1.5, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_lossy_vertical() {
        let aspect = 2.0 / 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::Y,
        });
        assert_eq!((0.0, 1.0, 0.25, 0.75), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_horizontal() {
        let aspect = 1.0 / 2.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::X,
        });
        assert_eq!((0.25, 0.75, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_vertical() {
        let aspect = 1.0 / 2.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::Y,
        });
        assert_eq!((0.0, 1.0, -0.5, 1.5), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_horizontal() {
        let aspect = 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::X,
        });
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_vertical() {
        let aspect = 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Lossy {
            stretch_direction: Axis2::Y,
        });
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_contain() {
        let aspect = 2.0 / 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Contain);
        assert_eq!((-0.5, 1.5, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_contain() {
        let aspect = 1.0 / 2.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Contain);
        assert_eq!((0.0, 1.0, -0.5, 1.5), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_contain() {
        let aspect = 1.0;
        let cam = CameraOrtho::normalized(CameraNormalizeMode::Contain);
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn custom_camera_large_contain() {
        let aspect = 2.0 / 1.0;
        let camera_ortho_world_coordinates = CameraOrthoWorldCoordinates {
            left: 0.,
            right: 800.,
            bottom: 0.,
            top: 600.,
        };
        let cam = CameraOrtho::new(CameraNormalizeMode::Contain, camera_ortho_world_coordinates);
        assert_eq!((-200.0, 1000.0, 0.0, 600.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn flipped_y_lossy_vertical() {
        let aspect = 1.0 / 2.0;
        let cam = CameraOrtho {
            mode: CameraNormalizeMode::Contain,
            world_coordinates: CameraOrthoWorldCoordinates {
                left: 0.0,
                right: 1.0,
                top: 0.0,
                bottom: 1.0,
            },
            aspect_ratio_cache: 0.0,
        };
        assert_eq!((0.0, 1.0, 1.5, -0.5), cam.camera_offsets(aspect));
    }

    #[test]
    fn camera_square_contain() {
        let aspect = 1.0;
        let cam = CameraOrtho {
            mode: CameraNormalizeMode::Contain,
            world_coordinates: CameraOrthoWorldCoordinates {
                left: 0.0,
                right: 2.0,
                top: 2.0,
                bottom: 0.0,
            },
            aspect_ratio_cache: 0.0,
        };
        assert_eq!((0.0, 2.0, 0.0, 2.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn camera_large_contain() {
        let aspect = 2.0 / 1.0;
        let cam = CameraOrtho {
            mode: CameraNormalizeMode::Contain,
            world_coordinates: CameraOrthoWorldCoordinates {
                left: 0.0,
                right: 2.0,
                top: 2.0,
                bottom: 0.0,
            },
            aspect_ratio_cache: 0.0,
        };
        assert_eq!((-1.0, 3.0, 0.0, 2.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn camera_high_contain() {
        let aspect = 1.0 / 2.0;
        let cam = CameraOrtho {
            mode: CameraNormalizeMode::Contain,
            world_coordinates: CameraOrthoWorldCoordinates {
                left: 0.0,
                right: 2.0,
                top: 2.0,
                bottom: 0.0,
            },
            aspect_ratio_cache: 0.0,
        };
        assert_eq!((0.0, 2.0, -1.0, 3.0), cam.camera_offsets(aspect));
    }
}
