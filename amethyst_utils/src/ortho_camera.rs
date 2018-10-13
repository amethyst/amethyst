//! Provides a automatically resized orthographic camera.

use amethyst_core::cgmath::Ortho;
use amethyst_core::specs::{
    Component, DenseVecStorage, Join, ReadExpect, ReadStorage, System, WriteStorage,
};
use amethyst_core::Axis2;
use amethyst_renderer::{Camera, ScreenDimensions};

/// `Component` attached to the camera's entity that allows automatically adjusting the camera's matrix according
/// to preferences in the "mode" field.
/// It tries as much as possible to adjust the camera so that the world's coordinate (0, 0) is at the bottom left and
/// (1, 1) is at the top right of the window.
/// You must add the `CameraNormalOrthoSystem` to your dispatcher for this to take effect.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CameraNormalOrtho {
    /// How the camera's matrix is changed when the window's aspect ratio changes.
    /// See `CameraNormalizeMode` for more info.
    pub mode: CameraNormalizeMode,
}

impl CameraNormalOrtho {
    /// Returns the camera matrix offsets according to the internal mode.
    pub fn camera_offsets(&self, ratio: f32) -> (f32, f32, f32, f32) {
        self.mode.camera_offsets(ratio)
    }
}

impl Component for CameraNormalOrtho {
    type Storage = DenseVecStorage<Self>;
}

/// Settings that decide how to scale the camera's matrix when the aspect ratio changes.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CameraNormalizeMode {
    /// Using an aspect ratio of 1:1, tries to adjust the matrix values of the camera so
    /// that the direction opposite to the stretch_direction always have a world size of 1.
    ///
    /// This means that the direction opposite to stretch_direction
    /// will always be between 0.0 to 1.0 in world coordinates.
    /// Scene space can be lost on the specified stretch_direction however.
    ///
    /// Example:
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

    /// Scales the render dynamically to ensure no space is lost in the [0,1] range on any axis.
    /// In other words, this ensures that you can always at least see everything
    /// between the world coordinates (0, 0) up to (1, 1).
    ///
    /// If you have a non-default `Transform` on your camera,
    /// it will just translate those coordinates by the translation of the `Transform`.
    Contain,
}

impl CameraNormalizeMode {
    /// Get the camera matrix offsets according to the specified options.
    fn camera_offsets(&self, aspect_ratio: f32) -> (f32, f32, f32, f32) {
        match self {
            &CameraNormalizeMode::Lossy {
                ref stretch_direction,
            } => match stretch_direction {
                Axis2::X => CameraNormalizeMode::lossy_x(aspect_ratio),
                Axis2::Y => CameraNormalizeMode::lossy_y(aspect_ratio),
            },
            &CameraNormalizeMode::Contain => {
                if aspect_ratio > 1.0 {
                    CameraNormalizeMode::lossy_x(aspect_ratio)
                } else if aspect_ratio < 1.0 {
                    CameraNormalizeMode::lossy_y(aspect_ratio)
                } else {
                    (0.0, 1.0, 0.0, 1.0)
                }
            }
        }
    }

    fn lossy_x(aspect_ratio: f32) -> (f32, f32, f32, f32) {
        let offset = (aspect_ratio - 1.0) / 2.0;
        (-offset, 1.0 + offset, 0.0, 1.0)
    }

    fn lossy_y(aspect_ratio: f32) -> (f32, f32, f32, f32) {
        let offset = (1.0 / aspect_ratio - 1.0) / 2.0;
        (0.0, 1.0, -offset, 1.0 + offset)
    }
}

impl Default for CameraNormalizeMode {
    fn default() -> Self {
        CameraNormalizeMode::Contain
    }
}

/// System that automatically changes the camera matrix according to the settings in
/// the `CameraNormalOrtho` attached to the camera entity.
#[derive(Default)]
pub struct CameraNormalOrthoSystem {
    aspect_ratio_cache: f32,
}

impl<'a> System<'a> for CameraNormalOrthoSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        WriteStorage<'a, Camera>,
        ReadStorage<'a, CameraNormalOrtho>,
    );
    fn run(&mut self, (dimensions, mut cameras, ortho_cameras): Self::SystemData) {
        let aspect = dimensions.aspect_ratio();
        if aspect != self.aspect_ratio_cache {
            self.aspect_ratio_cache = aspect;

            for (mut camera, ortho_camera) in (&mut cameras, &ortho_cameras).join() {
                let offsets = ortho_camera.camera_offsets(aspect);

                // Find the previous near and far would require
                // solving a linear system of two equation from
                // https://docs.rs/cgmath/0.16.1/src/cgmath/projection.rs.html#246-278
                camera.proj = Ortho {
                    left: offsets.0,
                    right: offsets.1,
                    bottom: offsets.2,
                    top: offsets.3,
                    near: 0.1,
                    far: 2000.0,
                }.into();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Axis2;
    use ortho_camera::{CameraNormalOrtho, CameraNormalizeMode};

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
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::X,
            },
        };
        assert_eq!((-0.5, 1.5, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_lossy_vertical() {
        let aspect = 2.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::Y,
            },
        };
        assert_eq!((0.0, 1.0, 0.25, 0.75), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_horizontal() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::X,
            },
        };
        assert_eq!((0.25, 0.75, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_vertical() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::Y,
            },
        };
        assert_eq!((0.0, 1.0, -0.5, 1.5), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_horizontal() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::X,
            },
        };
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_vertical() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {
                stretch_direction: Axis2::Y,
            },
        };
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_contain() {
        let aspect = 2.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Contain,
        };
        assert_eq!((-0.5, 1.5, 0.0, 1.0), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_contain() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Contain,
        };
        assert_eq!((0.0, 1.0, -0.5, 1.5), cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_contain() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Contain,
        };
        assert_eq!((0.0, 1.0, 0.0, 1.0), cam.camera_offsets(aspect));
    }
}
