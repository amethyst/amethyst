use super::base_3d::*;
use crate::{mtl::TexAlbedo, skinning::JointCombined};
use rendy::{
    hal::Backend,
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

pub type DrawFlatDesc<B, N> = DrawBase3DDesc<B, N, FlatPassDef>;
pub type DrawFlat<B, N> = DrawBase3D<B, N, FlatPassDef>;
pub type DrawFlatTransparentDesc<B, N> = DrawBase3DTransparentDesc<B, N, FlatPassDef>;
pub type DrawFlatTransparent<B, N> = DrawBase3DTransparent<B, N, FlatPassDef>;
