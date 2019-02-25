mod flat;
mod util;

pub use self::{
    flat::*,
};

use rendy::shader::{ShaderKind, SourceLanguage, StaticShaderInfo};

lazy_static::lazy_static! {
    static ref BASIC_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/basic.glsl"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    );

    static ref FLAT_FRAGMEN: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/flat.glsl"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref SKINNED_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/skinned.glsl"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );
}
