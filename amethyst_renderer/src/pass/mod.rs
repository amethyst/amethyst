//! Different kinds of render passes.
//
pub use self::flat::*;
pub use self::pbm::*;
pub use self::shaded::*;

mod flat;
mod pbm;
mod shaded;
mod skinning;
mod util;
mod shaded_util;
