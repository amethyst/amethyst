pub use gfx::preset::blend::{ALPHA, REPLACE};
pub use gfx_core::state::{Blend, BlendChannel, BlendValue, ColorMask, Equation, Factor};

use amethyst_core::specs::prelude::Component;
use amethyst_core::specs::storage::NullStorage;

/// Transparent mesh component
#[derive(Clone, Debug, Default)]
pub struct Transparent;

impl Component for Transparent {
    type Storage = NullStorage<Self>;
}
