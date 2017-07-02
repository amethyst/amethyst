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
use light::Light;
use mesh::Mesh;
use mtl::Material;
use rayon::slice::Iter;

/// Immutable parallel iterator of lights.
pub type Lights<'l> = Iter<'l, Light>;

/// Immutable parallel iterator of models.
pub type Models<'l> = Iter<'l, Model>;

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

    /// Iterates through all stored lights in parallel.
    pub fn par_iter_lights(&self) -> Lights {
        use rayon::prelude::*;
        self.lights.par_iter()
    }

    /// Iterates through all stored models in parallel.
    pub fn par_iter_models(&self) -> Models {
        use rayon::prelude::*;
        self.models.par_iter()
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub material: Material,
    pub mesh: Mesh,
}
