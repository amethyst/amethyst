use amethyst_core::Axis2;
use amethyst_core::cgmath::Ortho;
use amethyst_core::specs::{Component, DenseVecStorage, Join, System, ReadExpect, ReadStorage, WriteStorage};
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
    pub fn camera_offsets(&self, ratio: f32) -> (f32,f32,f32,f32) {
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
    /// If you use the default `Transform` (position = 0, 0, 0),
    /// this means that the direction opposite to stretch_direction 
    /// will go from 0.0 to 1.0 in world coordinates.
    /// Scene space can be lost on the specified stretch_direction.
    Lossy {stretch_direction: Axis2},
    
    /// Scales the render dynamically to ensure no space is lost in the [0,1] range on any axis.
    /// In other words, this ensures that you can always at least see everything 
    /// between the world coordinates (0, 0) up to (1, 1).
    /// If you have a non-default `Transform` on your camera,
    /// it will just translate those coordinates by the translation of the `Transform`.
    Shrink,
}

impl CameraNormalizeMode {
    /// Get the camera matrix offsets according to the specified options.
    fn camera_offsets(&self, aspect_ratio: f32) -> (f32,f32,f32,f32) {
        match self {
            &CameraNormalizeMode::Lossy {ref stretch_direction} => {
                match stretch_direction {
                    Axis2::X => {
                        CameraNormalizeMode::lossy_x(aspect_ratio)
                    },
                    Axis2::Y => {
                        CameraNormalizeMode::lossy_y(aspect_ratio)
                    },
                }
            },
            &CameraNormalizeMode::Shrink => {
                if aspect_ratio > 1.0 {
                    CameraNormalizeMode::lossy_x(aspect_ratio)
                } else if aspect_ratio < 1.0 {
                    CameraNormalizeMode::lossy_y(aspect_ratio)
                } else {
                    (0.0,1.0,0.0,1.0)
                }
            },
        }
    }
    
    fn lossy_x(aspect_ratio: f32) -> (f32,f32,f32,f32) {
        let offset = (aspect_ratio - 1.0) / 2.0;
        (-offset, 1.0 + offset, 0.0, 1.0)
    }

    fn lossy_y(aspect_ratio: f32) -> (f32,f32,f32,f32) {
        let offset = (1.0 / aspect_ratio - 1.0) / 2.0;
        (0.0, 1.0, -offset, 1.0 + offset)
    }
}

impl Default for CameraNormalizeMode {
    fn default() -> Self {
        CameraNormalizeMode::Shrink
    }
}

/// System that automatically changes the camera matrix according to the settings in
/// the `CameraNormalOrtho` attached to the camera entity.
#[derive(Default)]
pub struct CameraNormalOrthoSystem {
    aspect_ratio_cache: f32,
}

impl<'a> System<'a> for CameraNormalOrthoSystem {
    type SystemData = (ReadExpect<'a, ScreenDimensions>, WriteStorage<'a, Camera>, ReadStorage<'a, CameraNormalOrtho>);
    fn run(&mut self, (dimensions, mut cameras, ortho_cameras): Self::SystemData) {
        let aspect = dimensions.aspect_ratio();
        if aspect != self.aspect_ratio_cache {
            self.aspect_ratio_cache = aspect;

            for (mut camera, ortho_camera) in (&mut cameras, &ortho_cameras).join() {
                let offsets = ortho_camera.camera_offsets(aspect);
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
    use ortho_camera::{CameraNormalOrtho, CameraNormalizeMode};
    use super::Axis2;

    #[test]
    fn normal_camera_large_lossy_horizontal() {
        let aspect = 2.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::X},
        };
        assert_eq!((-0.5,1.5,0.0,1.0) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_lossy_vertical() {
        let aspect = 2.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::Y},
        };
        assert_eq!((0.0,1.0,0.25,0.75) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_horizontal() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::X},
        };
        assert_eq!((0.25,0.75,0.0,1.0) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_lossy_vertical() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::Y},
        };
        assert_eq!((0.0,1.0,-0.5,1.5) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_horizontal() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::X},
        };
        assert_eq!((0.0,1.0,0.0,1.0) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_lossy_vertical() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Lossy {stretch_direction: Axis2::Y},
        };
        assert_eq!((0.0,1.0,0.0,1.0) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_large_shrink() {
        let aspect = 2.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Shrink,
        };
        assert_eq!((-0.5,1.5,0.0,1.0) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_high_shrink() {
        let aspect = 1.0 / 2.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Shrink,
        };
        assert_eq!((0.0,1.0,-0.5,1.5) ,cam.camera_offsets(aspect));
    }

    #[test]
    fn normal_camera_square_shrink() {
        let aspect = 1.0 / 1.0;
        let cam = CameraNormalOrtho {
            mode: CameraNormalizeMode::Shrink,
        };
        assert_eq!((0.0,1.0,0.0,1.0) ,cam.camera_offsets(aspect));
    }
}