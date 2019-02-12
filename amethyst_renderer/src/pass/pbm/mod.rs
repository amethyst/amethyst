pub use self::{interleaved::DrawPbm, separate::DrawPbmSeparate};

mod interleaved;
mod separate;

use crate::pass::util::TextureType;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/pbm.glsl");

static TEXTURES: [TextureType; 7] = [
    TextureType::Roughness,
    TextureType::Caveat,
    TextureType::Metallic,
    TextureType::AmbientOcclusion,
    TextureType::Emission,
    TextureType::Normal,
    TextureType::Albedo,
];
