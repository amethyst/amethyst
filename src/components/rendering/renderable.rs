use components::rendering::Mesh;
use components::rendering::Texture;
use ecs::{Component, VecStorage};

#[derive(Clone)]
pub struct Renderable {
    pub mesh: Mesh,
    pub ka: Texture,
    pub kd: Texture,
}

impl Renderable {
    pub fn new(mesh: Mesh, ka: Texture, kd: Texture) -> Renderable {
        Renderable {
            mesh: mesh,
            ka: ka,
            kd: kd,
        }
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}
