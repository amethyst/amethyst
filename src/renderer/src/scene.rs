//! A fully renderable scene.

use {Light, Mesh};
use fnv::FnvHashMap as HashMap;

/// Scene struct.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    lights: HashMap<String, Light>,
    meshes: HashMap<String, Mesh>,
}

impl Scene {
    /// Adds a light source to the scene.
    pub fn add_light<N: Into<String>, L: Into<Light>>(&mut self, name: N, light: L) {
        self.lights.insert(name.into(), light.into());
    }

    /// Adds a mesh to the scene.
    pub fn add_mesh<N: Into<String>>(&mut self, name: N, mesh: Mesh) {
        self.meshes.insert(name.into(), mesh);
    }

    /// Removes a light source from the scene.
    pub fn remove_light(&mut self, name: &str) -> Option<Light> {
        self.lights.remove(name)
    }

    /// Returns an immutable reference to all lights in the scene.
    pub fn lights(&self) -> &HashMap<String, Light> {
        &self.lights
    }
    
    /// Returns an immutable reference to all meshes in the scene.
    pub fn meshes(&self) -> &HashMap<String, Mesh> {
        &self.meshes
    }
}
