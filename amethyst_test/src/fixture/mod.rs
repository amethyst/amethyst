//! Contains effects and assertions to test capabilities of an Amethyst application.
//!
//! Technically all effect and assertion functions can be moved here if it is useful for external
//! crates.

#[cfg(feature = "animation")]
pub use self::{
    material_animation_fixture::MaterialAnimationFixture,
    sprite_render_animation_fixture::SpriteRenderAnimationFixture,
};

#[cfg(feature = "animation")]
mod material_animation_fixture;
#[cfg(feature = "animation")]
mod sprite_render_animation_fixture;
