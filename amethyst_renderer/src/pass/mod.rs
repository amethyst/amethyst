//! Different kinds of render passes.

pub use self::blit::BlitBuffer;
pub use self::clear::ClearTarget;
pub use self::flat::DrawFlat;
pub use self::shaded::DrawShaded;

mod blit;
mod clear;
mod flat;
mod shaded;