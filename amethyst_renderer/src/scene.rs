//! A fully renderable scene.
//!
//! # Example
//!
//! ```rust
//! let mut scene = Scene::default();
//! scene.add_light(PointLight::default());
//! scene.add_mesh(sphere, material);
//! ```

use cam::Camera;
use fnv::FnvHashMap as HashMap;
use light::Light;
use mesh::Mesh;
use mtl::Material;
use std::collections::hash_map::Values;

/// Immutable slice iterator of lights.
pub type LightIter<'l> = Values<'l, String, Light>;

/// Immutable slice iterator of meshes.
pub type MeshIter<'m> = Values<'m, String, Mesh>;

/// Collection of lights and meshes to render.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    camera: Vec<Camera>,
    lights: HashMap<String, Light>,
    mats: HashMap<String, Material>,
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

    /// Removes a mesh from the scene.
    pub fn remove_mesh(&mut self, name: &str) -> Option<Mesh> {
        self.meshes.remove(name)
    }

    /// Returns an immutable reference to all lights in the scene.
    pub fn iter_lights(&self) -> LightIter {
        self.lights.values()
    }

    /// Returns an immutable reference to all meshes in the scene.
    pub fn iter_meshes(&self) -> MeshIter {
        self.meshes.values()
    }
}
