//! Physically renderable component.

use ecs::{Component, VecStorage};
use ecs::components::rendering::{Mesh, Texture};

/// A `Component` that can be attached to an ECS `Entity` to render it onscreen.
///
/// It combines geometry and various textures used in lighting calculations
/// to represent an entity on the screen. Every `Renderable`, `Transform`
/// pair attached to an entity inside the ECS `World` is rendered by the
/// `GfxDevice::render_world` method.
#[derive(Clone)]
pub struct Renderable {
    /// Mesh asset to be renderered.
    pub mesh: Mesh,
    /// Applied during ambient lighting calculations.
    pub ambient: Texture,
    /// Applied during diffuse lighting calculations.
    pub diffuse: Texture,
    /// Applied during specular lighting calculations.
    pub specular: Texture,
    /// Shininess of the object's surface.
    pub specular_exponent: f32,
}

impl Renderable {
    /// Creates a new renderable.
    pub fn new(mesh: Mesh,
               ambient: Texture,
               diffuse: Texture,
               specular: Texture,
               specular_exponent: f32)
               -> Renderable {

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
