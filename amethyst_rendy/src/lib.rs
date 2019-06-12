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

#[cfg(feature = "test-support")]
mod render_test_bundle;

pub use camera::Camera;
pub use formats::{
    mesh::MeshPrefab,
    texture::{ImageFormat, TexturePrefab},
};
pub use mtl::{Material, MaterialDefaults};
pub use sprite::{Sprite, SpriteRender, SpriteSheet, SpriteSheetFormat};
pub use system::{GraphCreator, RenderingSystem};
pub use transparent::Transparent;
pub use types::{Backend, Mesh, Texture};
pub use util::{simple_shader_set, ChangeDetection};

#[cfg(feature = "test-support")]
pub use render_test_bundle::{RenderEmptyBundle, RenderTestBundle};

pub use rendy::{
    factory::Factory,
    graph::{
        render::{RenderGroupDesc, SubpassBuilder},
        GraphBuilder,
    },
    hal::{format::Format, image::Kind},
};

pub mod loaders {
    pub use rendy::texture::palette::{load_from_linear_rgba, load_from_srgb, load_from_srgba};
}

static_assertions::assert_cfg!(
    any(feature = "metal", feature = "vulkan", feature = "empty"),
    "You must specify a graphical backend feature: 'empty' or 'vulkan' or 'metal'"
);
