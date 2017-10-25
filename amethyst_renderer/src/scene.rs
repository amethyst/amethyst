//! A fully renderable scene.

use cgmath::Matrix4;

use cam::Camera;
use light::Light;
use mesh::Mesh;
use mtl::Material;
use color::Rgba;

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

    /// Get all models on scene
    pub fn models(&self) -> &[Model] {
        &self.models
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
