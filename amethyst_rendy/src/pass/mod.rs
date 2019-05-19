mod base_3d;
mod debug_lines;
mod flat;
mod flat2d;
mod pbr;
mod shaded;
mod skybox;

pub use self::{base_3d::*, debug_lines::*, flat::*, flat2d::*, pbr::*, shaded::*, skybox::*};

use rendy::{hal::pso::ShaderStageFlags, shader::SpirvShader};

lazy_static::lazy_static! {
    static ref POS_TEX_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_tex.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref POS_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_tex_skin.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref POS_NORM_TEX_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_norm_tex.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref POS_NORM_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_norm_tex_skin.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref POS_NORM_TANG_TEX_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_norm_tang_tex.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref POS_NORM_TANG_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/pos_norm_tang_tex_skin.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref FLAT_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/flat.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref SHADED_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/shaded.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref PBR_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/pbr.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref SPRITE_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/sprite.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref SPRITE_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/sprite.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref SKYBOX_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/skybox.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref SKYBOX_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/skybox.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref DEBUG_LINES_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/vertex/debug_lines.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref DEBUG_LINES_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../../compiled/fragment/debug_lines.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );
}
