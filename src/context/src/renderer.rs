extern crate amethyst_renderer;
extern crate gfx_device_gl;

use self::amethyst_renderer::{Layer, Target, Camera, Fragment, Light};
use video_context::{VideoContext, DisplayConfig};

pub struct Renderer {
    video_context: VideoContext,
}

impl Renderer {
    pub fn new(display_config: DisplayConfig) -> Renderer {
        let video_context = VideoContext::new(display_config);
        Renderer {
            video_context: video_context,
        }
    }

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

    pub fn add_fragment(&mut self, scene_name: String, mesh: Mesh) -> usize {
        match self.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let scene = frame.scenes.get_mut(&scene_name).unwrap();
                if let FragmentImpl::OpenGL{ fragment } = mesh.fragment_impl {
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
    pub fn delete_fragment(&mut self, scene_name: String, idx: usize) {
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

    pub fn mut_video_context(&mut self) -> &mut VideoContext {
        &mut self.video_context
    }

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

// An enum with variants representing
// Fragments compatible with respective backends
#[allow(dead_code)]
pub enum FragmentImpl {
    OpenGL {
        fragment: Fragment<gfx_device_gl::Resources>,
    },
    Direct3D {
        // stub
    },
}

pub struct Mesh {
    fragment_impl: FragmentImpl,
}
