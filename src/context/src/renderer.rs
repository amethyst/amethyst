//! This module provides a frontend for
//! `amethyst_renderer`.

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;

pub use self::gfx::tex::Kind;

use self::amethyst_renderer::{Layer, Scene, Target, Camera, Light};
use video_context::VideoContext;

/// A wraper around `VideoContext` required to
/// hide all platform specific code from the user.
pub struct Renderer {
    video_context: VideoContext,
}

impl Renderer {
    /// Create a new `Renderer` from `DisplayConfig`.
    pub fn new(video_context: VideoContext) -> Renderer {
        Renderer {
            video_context: video_context,
        }
    }

    /// Set the rendering pipeline to be used.
    pub fn set_pipeline(&mut self, pipeline: Vec<Layer>) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.layers = pipeline;
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.targets.insert(name.into(), target);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.targets.remove(name.into());
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    /// Add an empty `Scene`.
    pub fn add_scene(&mut self, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = Scene::new();
                frame.scenes.insert(name.into(), scene);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }
    /// Delete a `Scene`.
    pub fn delete_scene(&mut self, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.scenes.remove(name.into());
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Fragment` to the scene with name `scene_name`.
    /// Return the index of the added `Fragment`.
    pub fn add_fragment(&mut self, scene_name: &str, fragment: Fragment) -> usize {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                if let FragmentImpl::OpenGL { fragment } = fragment.fragment_impl {
                    scene.fragments.push(fragment);
                }
                scene.fragments.len() - 1
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => 0,
        }
    }
    /// Get a mutable reference to the transform field of `Fragment` with index `idx`
    /// in scene `scene_name`.
    pub fn mut_fragment_transform(&mut self, scene_name: &str, idx: usize) -> Option<&mut [[f32; 4]; 4]> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                Some(&mut scene.fragments[idx].transform)
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => None,
        }
    }
    /// Delete `Fragment` with index `idx` in scene `scene_name`.
    pub fn delete_fragment(&mut self, scene_name: &str, idx: usize) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                scene.fragments.remove(idx);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Light` to the scene `scene_name`.
    /// Return the index of the added `Light`.
    pub fn add_light(&mut self, scene_name: &str, light: Light) -> usize {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                scene.lights.push(light);
                scene.lights.len() - 1
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => 0,
        }
    }
    /// Lookup `Light` in scene `scene_name` by index.
    pub fn mut_light(&mut self, scene_name: &str, idx:usize) -> Option<&mut Light> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                scene.lights.get_mut(idx)
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => None,
        }
    }
    /// Delete `Light` with index `idx` in scene `scene_name`.
    pub fn delete_light(&mut self, scene_name: &str, idx: usize) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(scene_name.into()).unwrap();
                scene.lights.remove(idx);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Camera`.
    pub fn add_camera(&mut self, camera: Camera, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.insert(name.into(), camera);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }
    /// Lookup a `Camera` by name.
    pub fn mut_camera(&mut self, name: &str) -> Option<&mut Camera> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.get_mut(name.into())
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => None,
        }
    }
    /// Delete a `Camera`.
    pub fn delete_camera(&mut self, name: &str) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.remove(name.into());
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }

    pub fn get_dimensions(&mut self) -> Option<(u32, u32)> {
        match self.video_context {
            VideoContext::OpenGL { ref window,
                                   .. } => {
                window.get_inner_size()
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => None,
        }
    }

    /// Get a mutable reference to `VideoContext`.
    pub fn mut_video_context(&mut self) -> &mut VideoContext {
        &mut self.video_context
    }

    /// Submit the `Frame` to `amethyst_renderer::Renderer`.
    pub fn submit(&mut self) {
        match self.video_context {
            VideoContext::OpenGL { ref window,
                                   ref mut renderer,
                                   ref frame,
                                   ref mut device,
                                   .. } => {
                renderer.submit(frame, device);
                window.swap_buffers().unwrap();
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                unimplemented!();
            },
            VideoContext::Null => (),
        }
    }
}

/// An enum with variants representing concrete
/// `Fragment` types compatible with different backends.
pub enum FragmentImpl {
    OpenGL {
        fragment: amethyst_renderer::Fragment<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Fragment` required to
/// hide all platform specific code from the user.
pub struct Fragment {
    pub fragment_impl: FragmentImpl,
}

/// An enum with variants representing concrete
/// `Texture` types compatible with different backends.
#[derive(Clone)]
pub enum TextureImpl {
    OpenGL {
        texture: amethyst_renderer::Texture<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Texture {
    pub texture_impl: TextureImpl,
}
