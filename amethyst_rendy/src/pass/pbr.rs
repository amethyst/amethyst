use rendy::{
    mesh::{AsVertex, Normal, Position, Tangent, TexCoord, VertexFormat},
    shader::SpirvShader,
};

use super::base_3d::*;
use crate::{mtl::FullTextureSet, skinning::JointCombined};

/// Implementation of `Base3DPassDef` for Physically-based (PBR) rendering pass.
#[derive(Debug)]
pub struct PbrPassDef;
impl Base3DPassDef for PbrPassDef {
    const NAME: &'static str = "Pbr";
    type TextureSet = FullTextureSet;
    fn vertex_shader() -> &'static SpirvShader {
        &super::POS_NORM_TANG_TEX_VERTEX
    }
    fn vertex_skinned_shader() -> &'static SpirvShader {
        &super::POS_NORM_TANG_TEX_SKIN_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &super::PBR_FRAGMENT
    }
    fn base_format() -> Vec<VertexFormat> {
        vec![
            Position::vertex(),
            Normal::vertex(),
            Tangent::vertex(),
            TexCoord::vertex(),
        ]
    }
    fn skinned_format() -> Vec<VertexFormat> {
        vec![
            Position::vertex(),
            Normal::vertex(),
            Tangent::vertex(),
            TexCoord::vertex(),
            JointCombined::vertex(),
        ]
    }
}

/// Describes a Physically-based (PBR) 3d Pass with lighting
pub type DrawPbrDesc<B> = DrawBase3DDesc<B, PbrPassDef>;
/// Draws a Physically-based (PBR) 3d Pass with lighting
pub type DrawPbr<B> = DrawBase3D<B, PbrPassDef>;
/// Describes a Physically-based (PBR) 3d Pass with lighting and transparency
pub type DrawPbrTransparentDesc<B> = DrawBase3DTransparentDesc<B, PbrPassDef>;
/// Draws a Physically-based (PBR) 3d Pass with lighting and transparency
pub type DrawPbrTransparent<B> = DrawBase3DTransparent<B, PbrPassDef>;
