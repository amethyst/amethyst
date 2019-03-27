// mod flat;
mod pbm;
mod util;
mod debug_lines;

pub use self::pbm::*;

use rendy::shader::{ShaderKind, SourceLanguage, StaticShaderInfo};

lazy_static::lazy_static! {
    static ref BASIC_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/basic.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    );

    static ref FLAT_FRAGMENT: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/flat.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref PBM_FRAGMENT: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/pbm.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref SKINNED_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/skinned.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref DEBUG_LINES_VERTEX: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/debug_lines.vert"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref DEBUG_LINES_GEOM: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/geometry/debug_lines.geom"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    static ref DEBUG_LINES_FRAG: StaticShaderInfo = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/debug_lines.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );
}
