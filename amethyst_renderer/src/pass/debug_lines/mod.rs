pub use self::interleaved::DebugLinesParams;
pub use self::interleaved::DrawDebugLines;

mod interleaved;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/debug_lines.glsl");
static GEOM_SRC: &[u8] = include_bytes!("../shaders/geometry/debug_lines.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/debug_lines.glsl");
