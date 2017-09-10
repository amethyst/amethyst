//! A fully renderable scene.

use rayon::prelude::*;
use rayon::slice::{Chunks, Iter};
use cgmath::Matrix4;

use cam::Camera;
use light::Light;
use mesh::Mesh;
use mtl::Material;
use color::Rgba;

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
    ambient_color: Rgba,
}

impl Scene {
    /// Adds a light source to the scene.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use amethyst_renderer::Scene;
    /// # use amethyst_renderer::light::PointLight;
    /// let mut scene = Scene::default();
    /// scene.add_light(PointLight::default());
    /// ```
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

    /// Set ambient color for the scene
    pub fn set_ambient_color(&mut self, color: Rgba) {
        self.ambient_color = color;
    }

    /// Get the ambient color for the scene
    pub fn ambient_color(&self) -> Rgba {
        self.ambient_color.clone()
    }

    /// Get all lights on scene
    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    /// Iterates through all stored lights in parallel.
    pub fn par_iter_lights(&self) -> Lights {
        self.lights.par_iter()
    }

    /// Iterates through all stored lights in parallel in chunks.
    pub fn par_chunks_lights(&self, count: usize) -> LightsChunks {
        let size = self.lights.len();
        if size > 0 {
            self.lights.par_chunks((size - 1) / count + 1)
        } else {
            self.lights.par_chunks(1)
        }
    }

    /// Iterates through all stored models in parallel.
    pub fn par_iter_models(&self) -> Models {
        self.models.par_iter()
    }

    /// Iterates through all stored models in parallel in chunks.
    pub fn par_chunks_models(&self, count: usize) -> ModelsChunks {
        let size = self.models.len();
        if size > 0 {
            self.models.par_chunks((size - 1) / count + 1)
        } else {
            self.models.par_chunks(1)
        }
    }

    /// Returns the active camera in the scene.
    ///
    /// TODO: Render to multiple viewports with possibly different cameras.
    pub fn active_camera(&mut self) -> Option<&mut Camera> {
        self.cameras.first_mut()
    }

    /// Remove all objects from `Scene`
    pub fn clear(&mut self) {
        self.models.clear();
        self.lights.clear();
        self.cameras.clear();
    }
}

/// A renderable object in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// Material properties of the model.
    pub material: Material,
    /// Physical geometry of the model.
    pub mesh: Mesh,
    /// Model matrix.
    pub pos: Matrix4<f32>,
}
