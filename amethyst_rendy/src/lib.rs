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
//! * [pass::flat2d::DrawFlat2DDesc]
//! * [pass::flat2d::DrawFlat2DTransparentDesc]
//! * [pass::pbr::DrawPbrDesc]
//! * [pass::flat::DrawFlatDesc]
//! * [pass::shaded::DrawShadedDesc]
//! * [pass::skybox::DrawSkyboxDesc]
//! * [pass::debug_lines::DrawDebugLinesDesc]
//!
//! ## Systems
//! * [system::RenderingSystem]
//! * [visibility::VisibilitySortingSystem]
//! * [sprite_visibility::SpriteVisibilitySortingSystem]
//!
//! ## Components
//! * [camera::Camera]
//! * [sprite_visibility::SpriteVisibility]
//! * [visibility::Visibility]
//! * [visibility::BoundingSphere]
//! * [debug_drawing::DebugLinesComponent]
//! * [light::Light]
//! * [resources::Tint]
//! * [skinning::JointTransforms]
//! * [sprite::SpriteRender]

#![allow(dead_code)]
#![allow(unused_variables)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate amethyst_derive;

#[macro_use]
extern crate shred_derive;

#[macro_use]
mod macros;

#[doc(inline)] pub use palette;
#[doc(inline)] pub use rendy;

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

#[doc(inline)] pub use formats::{mesh::MeshPrefab, texture::TexturePrefab};
#[doc(inline)] pub use mtl::{Material, MaterialDefaults};
#[doc(inline)] pub use sprite::{Sprite, SpriteRender, SpriteSheet};
#[doc(inline)] pub use system::{GraphCreator, RenderingSystem};
#[doc(inline)] pub use types::{Backend, Mesh, Texture};
#[doc(inline)] pub use util::{simple_shader_set, ChangeDetection};

pub mod loaders {
    //! Loaders re-exported from `rendy` for loading the most common image types as textures.

    pub use rendy::texture::palette::{load_from_linear_rgba, load_from_srgb, load_from_srgba};
}
