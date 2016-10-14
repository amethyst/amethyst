//! This module provides a frontend for
//! `amethyst_renderer`.

macro_rules! unwind_video_context_mut {
    ($variable:expr, $field1:ident, $expr_field:expr, $expr_null:expr) => {
        match $variable {
            VideoContext::OpenGL {
                ref mut $field1,
                ..
            } => $expr_field,
            #[cfg(windows)]
            VideoContext::Direct3D { } => unimplemented!(),
            VideoContext::Null => $expr_null,
        }
    };
}

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;

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
        Renderer { video_context: video_context }
    }

    /// Set the rendering pipeline to be used.
    pub fn set_pipeline(&mut self, pipeline: Vec<Layer>) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.layers = pipeline;
                                  },
                                  ())
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.targets.insert(name.into(), target);
                                  },
                                  ())
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.targets.remove(name.into());
                                  },
                                  ())
    }

    /// Add an empty `Scene`.
    pub fn add_scene(&mut self, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = Scene::new();
                                      frame.scenes.insert(name.into(), scene);
                                  },
                                  ())
    }
    /// Delete a `Scene`.
    pub fn delete_scene(&mut self, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.scenes.remove(name);
                                  },
                                  ())
    }

    /// Add a `Fragment` to the scene with name `scene_name`.
    /// Return the index of the added `Fragment`.
    pub fn add_fragment(&mut self, scene_name: &str, fragment: Fragment) -> Option<usize> {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                match fragment.fragment_impl {
                    FragmentImpl::OpenGL { fragment } => {
                        scene.fragments.push(fragment);
                        Some(scene.fragments.len() - 1)
                    }
                    #[cfg(windows)]
                    FragmentImpl::Direct3D {} => unimplemented!(),
                    FragmentImpl::Null => None,
                }
            }
            #[cfg(windows)]
            VideoContext::Direct3D {} => {
                unimplemented!();
            }
            VideoContext::Null => None,
        }
    }
    /// Get a mutable reference to the transform field of `Fragment` with index `idx`
    /// in scene `scene_name`.
    pub fn mut_fragment_transform(&mut self, scene_name: &str, idx: usize) -> Option<&mut [[f32; 4]; 4]> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return None,
                                      };
                                      Some(&mut scene.fragments[idx].transform)
                                  },
                                  None)
    }
    /// Delete `Fragment` with index `idx` in scene `scene_name`.
    pub fn delete_fragment(&mut self, scene_name: &str, idx: usize) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return,
                                      };
                                      scene.fragments.remove(idx);
                                  },
                                  ())
    }

    // Return number of fragments in scene `scene_name`.
    pub fn num_fragments(&mut self, scene_name: &str) -> Option<usize> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return None,
                                      };
                                      Some(scene.fragments.len())
                                  },
                                  None)
    }

    /// Add a `Light` to the scene `scene_name`.
    /// Return the index of the added `Light`.
    pub fn add_light(&mut self, scene_name: &str, light: Light) -> Option<usize> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return None,
                                      };
                                      scene.lights.push(light);
                                      Some(scene.lights.len() - 1)
                                  },
                                  None)
    }
    /// Lookup `Light` in scene `scene_name` by index.
    pub fn mut_light(&mut self, scene_name: &str, idx: usize) -> Option<&mut Light> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return None,
                                      };
                                      scene.lights.get_mut(idx)
                                  },
                                  None)
    }
    /// Delete `Light` with index `idx` in scene `scene_name`.
    pub fn delete_light(&mut self, scene_name: &str, idx: usize) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return,
                                      };
                                      scene.lights.remove(idx);
                                  },
                                  ())
    }

    // Return number of lights in scene `scene_name`.
    pub fn num_lights(&mut self, scene_name: &str) -> Option<usize> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      let scene = match frame.scenes.get_mut(scene_name.into()) {
                                          Some(scene) => scene,
                                          None => return None,
                                      };
                                      Some(scene.lights.len())
                                  },
                                  None)
    }

    /// Add a `Camera`.
    pub fn add_camera(&mut self, camera: Camera, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.cameras.insert(name.into(), camera);
                                  },
                                  ())
    }
    /// Lookup a `Camera` by name.
    pub fn mut_camera(&mut self, name: &str) -> Option<&mut Camera> {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.cameras.get_mut(name.into())
                                  },
                                  None)
    }
    /// Delete a `Camera`.
    pub fn delete_camera(&mut self, name: &str) {
        unwind_video_context_mut!(self.video_context,
                                  frame,
                                  {
                                      frame.cameras.remove(name.into());
                                  },
                                  ())
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        match self.video_context {
            VideoContext::OpenGL { ref window, .. } => window.get_inner_size(),
            #[cfg(windows)]
            VideoContext::Direct3D {} => unimplemented!(),
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
            VideoContext::OpenGL { ref window, ref mut device, ref mut renderer, ref frame } => {
                renderer.submit(frame, device);
                window.swap_buffers().unwrap();
            }
            #[cfg(windows)]
            VideoContext::Direct3D {} => unimplemented!(),
            VideoContext::Null => (),
        }
    }
}

/// An enum with variants representing concrete
/// `Fragment` types compatible with different backends.
pub enum FragmentImpl {
    OpenGL { fragment: amethyst_renderer::Fragment<gfx_device_gl::Resources>, },
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
