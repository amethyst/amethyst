//! A fully renderable scene.
//!
//! # Example
//!
//! ```rust
//! # use amethyst_renderer::Scene;
//! # use amethyst_renderer::light::PointLight;
//! let mut scene = Scene::default();
//! scene.add_light("light", PointLight::default());
//! ```

use cam::Camera;
use cgmath::Matrix4;
use light::Light;
use mesh::Mesh;
use mtl::Material;
use rayon::slice::{Chunks, Iter};

/// Immutable parallel iterator of lights.
pub type Lights<'l> = Iter<'l, Light>;

/// Immutable parallel iterator of models.
pub type Models<'l> = Iter<'l, Model>;

/// Immutable parallel iterator of models.
pub type ModelsChunks<'l> = Chunks<'l, Model>;

/// Immutable parallel iterator of models.
pub type LightsChunks<'l> = Chunks<'l, Light>;

/// Collection of lights and meshes to render.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    cameras: Vec<Camera>,
    lights: Vec<Light>,
    models: Vec<Model>,
}

impl Scene {
    /// Adds a light source to the scene.
    pub fn add_light<L: Into<Light>>(&mut self, light: L) {
        self.lights.push(light.into());
    }

    /// Adds a mesh to the scene.
    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    /// Adds a camera to the scene.
    pub fn add_camera<C: Into<Camera>>(&mut self, camera: C) {
        self.cameras.push(camera.into());
    }

    /// Get all lights on scene
    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    /// Iterates through all stored lights in parallel.
    pub fn par_iter_lights(&self) -> Lights {
        use rayon::prelude::*;
        self.lights.par_iter()
    }

    /// Iterates through all stored lights in parallel in chunks.
    pub fn par_chunks_lights(&self, count: usize) -> LightsChunks {
        use rayon::prelude::*;
        let size = self.lights.len();
        self.lights.par_chunks(((size - 1) / count) + 1)
    }

    /// Iterates through all stored models in parallel.
    pub fn par_iter_models(&self) -> Models {
        use rayon::prelude::*;
        self.models.par_iter()
    }

    /// Iterates through all stored models in parallel in chunks.
    pub fn par_chunks_models(&self, count: usize) -> ModelsChunks {
        use rayon::prelude::*;
        let size = self.models.len();
        self.models.par_chunks(((size - 1) / count) + 1)
    }

    /// Active camera
    /// TODO: Render to multiple viewports with possibly different cameras
    pub fn active_camera(&self) -> Option<&Camera> {
        self.cameras.first()
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub material: Material,
    pub mesh: Mesh,
    pub pos: Matrix4<f32>,
}
