pub use self::interleaved::DrawSprite;

mod interleaved;

use pass::util::TextureType;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/sprite.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/sprite.glsl");

static TEXTURES: [TextureType; 1] = [TextureType::Albedo];
