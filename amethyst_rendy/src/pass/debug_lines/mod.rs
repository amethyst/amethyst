pub use self::interleaved::{DebugLinesParams, DrawDebugLines};

mod interleaved;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/debug_lines.vert");
static GEOM_SRC: &[u8] = include_bytes!("../shaders/geometry/debug_lines.geom");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/debug_lines.frag");
