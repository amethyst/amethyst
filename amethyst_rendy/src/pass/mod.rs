mod flat;
mod pbm;
mod util;

pub use self::{flat::*, pbm::*};

use rendy::shader::{ShaderKind, SourceLanguage, StaticShaderInfo};

lazy_static::lazy_static! {
    static ref BASIC_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/basic.glsl"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    );

    static ref FLAT_FRAGMENT: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/flat.glsl"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref PBM_FRAGMENT: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/pbm.glsl"),
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
