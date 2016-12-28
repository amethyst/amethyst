use components::rendering::Mesh;
use components::rendering::Texture;
use ecs::{Component, VecStorage};

#[derive(Clone)]
/// This struct is a `Component`, which combines geometry and ka, kd textures.
/// Every `Renderable`, `Transform` pair attached to an entity inside the `World`
/// is rendered by `GfxDevice::render_world` method.
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
