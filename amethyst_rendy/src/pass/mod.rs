mod base_3d;
mod debug_lines;
mod flat;
mod flat2d;
mod pbr;
mod shaded;
mod skybox;

pub use self::{base_3d::*, debug_lines::*, flat::*, flat2d::*, pbr::*, shaded::*, skybox::*};

use rendy::shader::{ShaderKind, SourceLanguage, SpirvShader, StaticShaderInfo};

lazy_static::lazy_static! {
    static ref POS_TEX_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/pos_tex.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref POS_TEX_SKIN_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/pos_tex_skin.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
    static ref POS_NORM_TANG_TEX_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/pos_norm_tang_tex.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref POS_NORM_TANG_TEX_SKIN_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/pos_norm_tang_tex_skin.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FLAT_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/flat.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADED_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/shaded.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref PBR_SKINNED_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/pbr_skinned.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref PBR_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/pbr.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SPRITE_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/sprite.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SPRITE_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/sprite.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SKYBOX_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/skybox.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SKYBOX_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/skybox.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref DEBUG_LINES_VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex/debug_lines.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref DEBUG_LINES_FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment/debug_lines.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
}
