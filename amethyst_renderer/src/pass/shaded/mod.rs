pub use self::{interleaved::DrawShaded, separate::DrawShadedSeparate};

mod interleaved;
mod separate;

use crate::pass::util::TextureType;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/shaded.glsl");

static TEXTURES: [TextureType; 2] = [TextureType::Albedo, TextureType::Emission];
