use components::rendering::Mesh;
use components::rendering::Texture;
use ecs::{Component, VecStorage};

pub struct Renderable {
    pub mesh: Mesh,
    pub ka: Texture,
    pub kd: Texture,
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}
