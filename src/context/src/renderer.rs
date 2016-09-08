//! This module provides a frontend for
//! `amethyst_renderer`.
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
        Renderer {
            video_context: video_context,
        }
    }

    /// Set the rendering pipeline to be used.
    pub fn set_pipeline(&mut self, pipeline: Vec<Layer>) {
        self.video_context.frame.layers = pipeline;
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: &str) {
        self.video_context.frame.targets.insert(name.into(), target);
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: &str) {
        self.video_context.frame.targets.remove(name.into());
    }

    /// Add an empty `Scene`.
    pub fn add_scene(&mut self, name: &str) {
        let scene = Scene::new();
        self.video_context.frame.scenes.insert(name.into(), scene);
    }
    /// Delete a `Scene`.
    pub fn delete_scene(&mut self, name: &str) {
        self.video_context.frame.scenes.remove(name);
    }

    /// Add a `Fragment` to the scene with name `scene_name`.
    /// Return the index of the added `Fragment`.
    pub fn add_fragment(&mut self, scene_name: &str, fragment: Fragment) -> Option<usize> {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return None,
        };
        scene.fragments.push(fragment.data);
        Some(scene.fragments.len() - 1)
    }
    /// Get a mutable reference to the transform field of `Fragment` with index `idx`
    /// in scene `scene_name`.
    pub fn mut_fragment_transform(&mut self, scene_name: &str, idx: usize) -> Option<&mut [[f32; 4]; 4]> {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return None,
        };
        Some(&mut scene.fragments[idx].transform)
    }
    /// Delete `Fragment` with index `idx` in scene `scene_name`.
    pub fn delete_fragment(&mut self, scene_name: &str, idx: usize) {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return,
        };
        scene.fragments.remove(idx);
    }

    /// Add a `Light` to the scene `scene_name`.
    /// Return the index of the added `Light`.
    pub fn add_light(&mut self, scene_name: &str, light: Light) -> Option<usize> {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return None,
        };
        scene.lights.push(light);
        Some(scene.lights.len() - 1)
    }
    /// Lookup `Light` in scene `scene_name` by index.
    pub fn mut_light(&mut self, scene_name: &str, idx:usize) -> Option<&mut Light> {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return None,
        };
        scene.lights.get_mut(idx)
    }
    /// Delete `Light` with index `idx` in scene `scene_name`.
    pub fn delete_light(&mut self, scene_name: &str, idx: usize) {
        let scene = match self.video_context.frame.scenes.get_mut(scene_name.into()) {
            Some(scene) => scene,
            None => return,
        };
        scene.lights.remove(idx);
    }

    /// Add a `Camera`.
    pub fn add_camera(&mut self, camera: Camera, name: &str) {
        self.video_context.frame.cameras.insert(name.into(), camera);
    }
    /// Lookup a `Camera` by name.
    pub fn mut_camera(&mut self, name: &str) -> Option<&mut Camera> {
        self.video_context.frame.cameras.get_mut(name.into())
    }
    /// Delete a `Camera`.
    pub fn delete_camera(&mut self, name: &str) {
        self.video_context.frame.cameras.remove(name.into());
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        self.video_context.window.get_inner_size()
    }

    /// Get a mutable reference to `VideoContext`.
    pub fn mut_video_context(&mut self) -> &mut VideoContext {
        &mut self.video_context
    }

    /// Submit the `Frame` to `amethyst_renderer::Renderer`.
    pub fn submit(&mut self) {
        let ref mut renderer = self.video_context.renderer;
        let ref frame = self.video_context.frame;
        let ref mut device = self.video_context.device;
        let ref window = self.video_context.window;

        renderer.submit(&frame, device);
        window.swap_buffers().unwrap();
    }
}

/// A wraper around `Fragment`
pub struct Fragment {
    pub data: amethyst_renderer::Fragment<gfx_device_gl::Resources>,
}
