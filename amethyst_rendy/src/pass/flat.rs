use super::base_3d::*;
use crate::{mtl::TexAlbedo, skinning::JointCombined, types::Backend};
use rendy::{
    mesh::{AsVertex, Position, TexCoord, VertexFormat},
    shader::SpirvShader,
};

#[derive(Debug)]
pub struct FlatPassDef;
impl<B: Backend> Base3DPassDef<B> for FlatPassDef {
    const NAME: &'static str = "Flat";
    type TextureSet = TexAlbedo;
    fn vertex_shader() -> &'static SpirvShader {
        &super::POS_TEX_VERTEX
    }
    fn vertex_skinned_shader() -> &'static SpirvShader {
        &super::POS_TEX_SKIN_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &super::FLAT_FRAGMENT
    }
    fn base_format() -> Vec<VertexFormat> {
        vec![Position::vertex(), TexCoord::vertex()]
    }
    fn skinned_format() -> Vec<VertexFormat> {
        vec![
            Position::vertex(),
            TexCoord::vertex(),
            JointCombined::vertex(),
        ]
    }
}

pub type DrawFlatDesc<B> = DrawBase3DDesc<B, FlatPassDef>;
pub type DrawFlat<B> = DrawBase3D<B, FlatPassDef>;
pub type DrawFlatTransparentDesc<B> = DrawBase3DTransparentDesc<B, FlatPassDef>;
pub type DrawFlatTransparent<B> = DrawBase3DTransparent<B, FlatPassDef>;
