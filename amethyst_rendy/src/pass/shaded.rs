use super::base_3d::*;
use crate::{
    mtl::{TexAlbedo, TexEmission},
    skinning::JointCombined,
    types::Backend,
};
use rendy::{
    mesh::{AsVertex, Normal, Position, Tangent, TexCoord, VertexFormat},
    shader::SpirvShader,
};

#[derive(Debug)]
pub struct ShadedPassDef;
impl<B: Backend> Base3DPassDef<B> for ShadedPassDef {
    const NAME: &'static str = "Shaded";
    type TextureSet = (TexAlbedo, TexEmission);
    fn vertex_shader() -> &'static SpirvShader {
        &super::POS_NORM_TANG_TEX_VERTEX
    }
    fn vertex_skinned_shader() -> &'static SpirvShader {
        &super::POS_NORM_TANG_TEX_SKIN_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &super::SHADED_FRAGMENT
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

pub type DrawShadedDesc<B, N> = DrawBase3DDesc<B, N, ShadedPassDef>;
pub type DrawShaded<B, N> = DrawBase3D<B, N, ShadedPassDef>;
pub type DrawShadedTransparentDesc<B, N> = DrawBase3DTransparentDesc<B, N, ShadedPassDef>;
pub type DrawShadedTransparent<B, N> = DrawBase3DTransparent<B, N, ShadedPassDef>;
