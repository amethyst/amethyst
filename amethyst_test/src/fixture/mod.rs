//! Contains effects and assertions to test capabilities of an Amethyst application.
//!
//! Technically all effect and assertion functions can be moved here if it is useful for external
//! crates.

pub use self::{
    material_animation_fixture::MaterialAnimationFixture,
    sprite_render_animation_fixture::SpriteRenderAnimationFixture,
};

mod material_animation_fixture;
mod sprite_render_animation_fixture;
