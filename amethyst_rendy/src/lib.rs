// this is temporary
// #![allow(dead_code)]
// #![allow(unused_variables)]

#[macro_use]
extern crate amethyst_derive;

#[macro_use]
extern crate shred_derive;

#[macro_use]
mod macros;

pub use palette;
pub use rendy;

pub mod pass;

pub mod batch;
pub mod camera;
pub mod debug_drawing;
pub mod error;
pub mod formats;
pub mod light;
pub mod mtl;
pub mod pipeline;
pub mod resources;
pub mod serde_shim;
pub mod shape;
pub mod skinning;
pub mod sprite;
pub mod sprite_visibility;
pub mod submodules;
pub mod system;
pub mod transparent;
pub mod types;
pub mod visibility;

pub mod pod;
pub mod util;

pub use formats::{mesh::MeshPrefab, texture::TexturePrefab};
pub use mtl::{Material, MaterialDefaults};
pub use sprite::{Sprite, SpriteRender, SpriteSheet};
pub use system::{GraphCreator, RenderingSystem};
pub use types::{Backend, Mesh, Texture};
pub use util::{simple_shader_set, ChangeDetection};

pub mod loaders {
    pub use rendy::texture::palette::{load_from_linear_rgba, load_from_srgb, load_from_srgba};
}
