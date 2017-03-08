//! A fully renderable scene.

use Mesh;
use fnv::FnvHashMap as HashMap;

/// Scene struct.
#[derive(Debug, Default)]
pub struct Scene {
    meshes: HashMap<String, Mesh>,
}

impl Scene {
    /// Returns an immutable reference to all the meshes in the scene.
    pub fn meshes(&self) -> &HashMap<String, Mesh> {
        &self.meshes
    }
}
