use super::base_3d::*;
use crate::{
    mtl::{TexAlbedo, TexEmission},
    skinning::JointCombined,
    types::Backend,
};
use rendy::{
    mesh::{AsVertex, Normal, Position, TexCoord, VertexFormat},
    shader::SpirvShader,
};

#[derive(Debug)]
pub struct ShadedPassDef;
impl<B: Backend> Base3DPassDef<B> for ShadedPassDef {
    const NAME: &'static str = "Shaded";
    type TextureSet = (TexAlbedo, TexEmission);
    fn vertex_shader() -> &'static SpirvShader {
        &super::POS_NORM_TEX_VERTEX
    }
    fn vertex_skinned_shader() -> &'static SpirvShader {
        &super::POS_NORM_TEX_SKIN_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &super::SHADED_FRAGMENT
    }
    fn base_format() -> Vec<VertexFormat> {
        vec![Position::vertex(), Normal::vertex(), TexCoord::vertex()]
    }
    fn skinned_format() -> Vec<VertexFormat> {
        vec![
            Position::vertex(),
            Normal::vertex(),
            TexCoord::vertex(),
            JointCombined::vertex(),
        ]
    }
}

pub type DrawShadedDesc<B> = DrawBase3DDesc<B, ShadedPassDef>;
pub type DrawShaded<B> = DrawBase3D<B, ShadedPassDef>;
pub type DrawShadedTransparentDesc<B> = DrawBase3DTransparentDesc<B, ShadedPassDef>;
pub type DrawShadedTransparent<B> = DrawBase3DTransparent<B, ShadedPassDef>;
