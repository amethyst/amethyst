//! Different kinds of render passes.
//
pub use self::flat::DrawFlat;
pub use self::pbm::DrawPbm;
pub use self::shaded::DrawShaded;

mod flat;
mod pbm;
mod shaded;
