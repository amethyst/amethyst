use gfx_device::assets::Mesh;
use gfx_device::assets::Texture;
use ecs::{Component, VecStorage};

pub struct Renderable {
    pub mesh: Mesh,
    pub ka: Texture,
    pub kd: Texture,
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}
