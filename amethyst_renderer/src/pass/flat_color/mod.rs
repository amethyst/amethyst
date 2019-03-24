pub use self::{interleaved::DrawFlatColor, separate::DrawFlatColorSeparate};

mod interleaved;
mod separate;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic_color.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/flat_color.glsl");
