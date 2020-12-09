use rendy::{
    mesh::{AsVertex, Position, TexCoord, VertexFormat},
    shader::SpirvShader,
};

use super::base_3d::*;
use crate::{mtl::TexAlbedo, skinning::JointCombined};

/// Implementation of `Base3DPassDef` to describe a flat 3D pass
#[derive(Debug)]
pub struct FlatPassDef;
impl Base3DPassDef for FlatPassDef {
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

/// Describes a Flat 3D pass
pub type DrawFlatDesc<B> = DrawBase3DDesc<B, FlatPassDef>;
/// Draws a Flat 3D pass
pub type DrawFlat<B> = DrawBase3D<B, FlatPassDef>;
/// Describes a Flat 3D pass with Transparency
pub type DrawFlatTransparentDesc<B> = DrawBase3DTransparentDesc<B, FlatPassDef>;
/// Draws a Flat 3D pass with transpency.
pub type DrawFlatTransparent<B> = DrawBase3DTransparent<B, FlatPassDef>;
