extern crate specs;

use components::rendering::Mesh;
use components::rendering::Texture;
use self::specs::{Component, VecStorage};

#[derive(Clone)]
/// A `Component` that can be attached to an ECS `Entity` to render it onscreen.
///
/// It combines geometry and various textures used in lighting calculations
/// to represent an entity on the screen. Every `Renderable`, `Transform`
/// pair attached to an entity inside the ECS `World` is rendered by the
/// `GfxDevice::render_world` method.
pub struct Renderable {
    /// The geometry that will be rendered
    pub mesh: Mesh,

    /// Texture used in ambient lighting calculations
    pub ambient: Texture,

    /// Texture used in diffuse lighting calculations
    pub diffuse: Texture,

    /// Texture used in specular lighting calculations
    pub specular: Texture,

    /// Specular exponent used in lighting calculations
    pub specular_exponent: f32,
}

impl Renderable {
    /// Creates a new renderable. You will probably want not use this directly.
    /// Instead, use the `AssetManager::create_renderable` function.
    pub fn new(mesh: Mesh,
               ambient: Texture,
               diffuse: Texture,
               specular: Texture,
               specular_exponent: f32) -> Renderable {

        Renderable {
            mesh: mesh,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
            specular_exponent: specular_exponent,
        }
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}
