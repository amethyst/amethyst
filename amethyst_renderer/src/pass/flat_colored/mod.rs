pub use self::{interleaved::DrawFlatColored, separate::DrawFlatColoredSeparate};

mod interleaved;
mod separate;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/flat_colored.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/flat_colored.glsl");
