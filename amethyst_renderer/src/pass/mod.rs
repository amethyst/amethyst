//! Different kinds of render passes.
//
pub use self::{
    debug_lines::*,
    flat::*,
    flat2d::*,
    flat_colored::*,
    pbm::*,
    shaded::*,
    skinning::set_skinning_buffers,
    skybox::*,
    util::{get_camera, set_vertex_args},
};

mod debug_lines;
mod flat;
mod flat2d;
mod flat_colored;
mod pbm;
mod shaded;
mod shaded_util;
mod skinning;
mod skybox;
mod util;
