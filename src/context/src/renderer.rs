//! This module provides a frontend for
//! `amethyst_renderer`.

extern crate amethyst_renderer;
extern crate gfx_device_gl;

use self::amethyst_renderer::{Layer, Target, Camera, Fragment, Light};
use video_context::{VideoContext, DisplayConfig};

/// A wraper around `VideoContext` required to
/// hide all platform specific code from the user.
pub struct Renderer {
    video_context: VideoContext,
}

impl Renderer {
    /// Create a new `Renderer` from `DisplayConfig`.
    pub fn new(display_config: DisplayConfig) -> Renderer {
        let video_context = VideoContext::new(display_config);
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
                // stub
            },
            VideoContext::Null => (),
        }
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: String) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.targets.insert(name, target);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }
    /// Lookup a rendering `Target` by name.
    pub fn mut_target(&mut self, name: String) -> Option<&mut Box<Target>> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.targets.get_mut(&name)
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                None
            },
            VideoContext::Null => None,
        }
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: String) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.targets.remove(&name);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Mesh` to the scene with name `scene_name`.
    /// Return the index of the added `Mesh`.
    pub fn add_mesh(&mut self, scene_name: String, mesh: Mesh) -> usize {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                if let FragmentImpl::OpenGL { fragment } = mesh.fragment_impl {
                    scene.fragments.push(fragment);
                }
                scene.fragments.len() - 1
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                0
            },
            VideoContext::Null => 0,
        }
    }
    /// Delete `Mesh` with index `idx` in scene `scene_name`.
    pub fn delete_mesh(&mut self, scene_name: String, idx: usize) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                scene.fragments.remove(idx);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Light` to the scene `scene_name`.
    /// Return the index of the added `Light`.
    pub fn add_light(&mut self, scene_name: String, light: Light) -> usize {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                scene.lights.push(light);
                scene.lights.len() - 1
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                0
            },
            VideoContext::Null => 0,
        }
    }
    /// Lookup `Light` in scene `scene_name` by index.
    pub fn mut_light(&mut self, scene_name: String, idx:usize) -> Option<&mut Light> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                scene.lights.get_mut(idx)
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                None
            },
            VideoContext::Null => None,
        }
    }
    /// Delete `Light` with index `idx` in scene `scene_name`.
    pub fn delete_light(&mut self, scene_name: String, idx: usize) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                scene.lights.remove(idx);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }

    /// Add a `Camera`.
    pub fn add_camera(&mut self, camera: Camera, name: String) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.insert(name, camera);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }
    /// Lookup a `Camera` by name.
    pub fn mut_camera(&mut self, name: String) -> Option<&mut Camera> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.get_mut(&name)
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                None
            },
            VideoContext::Null => None,
        }
    }
    /// Delete a `Camera`.
    pub fn delete_camera(&mut self, name: String) {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                frame.cameras.remove(&name);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
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
                // stub
            },
            VideoContext::Null => (),
        }
    }
}

/// An enum with variants representing concrete
/// `Fragment` types compatible with different backends.
#[allow(dead_code)]
pub enum FragmentImpl {
    OpenGL {
        fragment: Fragment<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Fragment` required to
/// hide all platform specific code from the user.
pub struct Mesh {
    fragment_impl: FragmentImpl,
}
