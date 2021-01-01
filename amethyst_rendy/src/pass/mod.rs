//! Passes and shaders implemented by amethyst

mod base_3d;
mod debug_lines;
mod flat;
mod flat2d;
mod pbr;
mod shaded;
mod skybox;

use rendy::{hal::pso::ShaderStageFlags, shader::SpirvShader};

pub use self::{base_3d::*, debug_lines::*, flat::*, flat2d::*, pbr::*, shaded::*, skybox::*};

lazy_static::lazy_static! {
    static ref POS_TEX_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_tex.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_tex_skin.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_NORM_TEX_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_norm_tex.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_NORM_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_norm_tex_skin.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_NORM_TANG_TEX_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_norm_tang_tex.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_NORM_TANG_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/pos_norm_tang_tex_skin.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FLAT_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/flat.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref SHADED_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/shaded.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref PBR_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/pbr.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref SPRITE_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/sprite.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref SPRITE_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/sprite.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref SKYBOX_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/skybox.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref SKYBOX_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/skybox.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref DEBUG_LINES_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/vertex/debug_lines.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref DEBUG_LINES_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/fragment/debug_lines.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}
