//! This implementation of the Amethyst Renderer utilizes the `rendy` crate, built on top of
//! `gfx-hal` to provide the building blocks for a AAA configurable rendering graph-based pipeline.
//!
//! As a general overview, this crate can be broken down as follows:
//!
//! ### Core
//!
//! ### Submodules
//!
//! ### Passes
//!
//! * [`DrawFlat2DDesc`](crate::pass::flat2d::DrawFlat2DDesc)
//! * [`DrawFlat2DTransparentDesc`](crate::pass::flat2d::DrawFlat2DTransparentDesc)
//! * [`DrawPbrDesc`](crate::pass::pbr::DrawPbrDesc)
//! * [`DrawFlatDesc`](crate::pass::flat::DrawFlatDesc)
//! * [`DrawShadedDesc`](crate::pass::shaded::DrawShadedDesc)
//! * [`DrawSkyboxDesc`](crate::pass::skybox::DrawSkyboxDesc)
//! * [`DrawDebugLinesDesc`](crate::pass::debug_lines::DrawDebugLinesDesc)
//!
//! ## Systems
//!
//! * [`RenderingSystem`](crate::system::RenderingSystem)
//! * [`VisibilitySortingSystem`](crate::visibility::VisibilitySortingSystem)
//! * [`SpriteVisibilitySortingSystem`](crate::sprite_visibility::SpriteVisibilitySortingSystem)
//!
//! ## Components
//!
//! * [`Camera`](camera::Camera)
//! * [`SpriteVisibility`](sprite_visibility::SpriteVisibility)
//! * [`Visibility`](visibility::Visibility)
//! * [`BoundingSphere`](visibility::BoundingSphere)
//! * [`DebugLinesComponent`](debug_drawing::DebugLinesComponent)
//! * [`Light`](light::Light)
//! * [`Tint`](resources::Tint)
//! * [`JointTransforms`](skinning::JointTransforms)
//! * [`SpriteRender`](sprite::SpriteRender)

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]
#![allow(unused_variables, dead_code)]

#[macro_use]
extern crate amethyst_derive;

#[macro_use]
mod macros;

#[doc(inline)]
pub use palette;
#[doc(inline)]
pub use rendy;

pub mod pass;

pub mod batch;
pub mod bundle;
pub mod camera;
pub mod debug_drawing;
pub mod error;
pub mod formats;
pub mod light;
pub mod mtl;
pub mod pipeline;
pub mod plugins;
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

#[doc(inline)]
pub use crate::{
    bundle::{RenderPlugin, RenderingBundle},
    camera::{ActiveCamera, Camera},
    formats::{
        mesh::MeshPrefab,
        texture::{ImageFormat, TexturePrefab},
    },
    mtl::{Material, MaterialDefaults},
    plugins::*,
    sprite::{Sprite, SpriteRender, SpriteSheet, SpriteSheetFormat},
    system::{GraphCreator, MeshProcessorSystem, RenderingSystem, TextureProcessorSystem},
    transparent::Transparent,
    types::{Backend, Mesh, Texture},
    util::{simple_shader_set, ChangeDetection},
};

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
    //! Loaders re-exported from `rendy` for loading the most common image types as textures.

    pub use rendy::texture::palette::{load_from_linear_rgba, load_from_srgb, load_from_srgba};
}
