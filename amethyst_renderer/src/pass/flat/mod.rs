pub use self::interleaved::DrawFlat;
pub use self::separate::DrawFlatSeparate;

mod interleaved;
mod separate;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic.glsl");
static VERT_SKIN_SRC: &[u8] = include_bytes!("../shaders/vertex/skinned.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/flat.glsl");

#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}
