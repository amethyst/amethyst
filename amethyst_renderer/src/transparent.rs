pub use gfx_core::state::{Blend, BlendChannel, ColorMask, Equation, Factor};

use amethyst_core::specs::{Component, NullStorage};

/// Transparent mesh component
#[derive(Clone, Debug, Default)]
pub struct Transparent;

impl Component for Transparent {
    type Storage = NullStorage<Self>;
}
