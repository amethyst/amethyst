//! Different kinds of render passes.
//
pub use self::flat::*;
pub use self::pbm::*;
pub use self::shaded::*;
pub use self::skinning::set_skinning_buffers;
pub use self::util::{get_camera, set_vertex_args};

mod flat;
mod pbm;
mod shaded;
mod shaded_util;
mod skinning;
mod util;
