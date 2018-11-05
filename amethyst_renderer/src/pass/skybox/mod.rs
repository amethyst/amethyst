pub use self::interleaved::DrawSkybox;

mod interleaved;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/skybox.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/skybox.glsl");
