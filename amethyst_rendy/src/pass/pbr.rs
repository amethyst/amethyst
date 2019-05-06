use super::base_3d::*;
use crate::{mtl::FullTextureSet, skinning::JointCombined};
use rendy::{
    hal::Backend,
    mesh::{AsVertex, Normal, Position, Tangent, TexCoord, VertexFormat},
    shader::SpirvShader,
};

#[derive(Debug)]
pub struct PbrPassDef;
impl<B: Backend> Base3DPassDef<B> for PbrPassDef {
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

pub type DrawPbrDesc<B, N> = DrawBase3DDesc<B, N, PbrPassDef>;
pub type DrawPbr<B, N> = DrawBase3D<B, N, PbrPassDef>;
pub type DrawPbrTransparentDesc<B, N> = DrawBase3DTransparentDesc<B, N, PbrPassDef>;
pub type DrawPbrTransparent<B, N> = DrawBase3DTransparent<B, N, PbrPassDef>;
