use crate::pass::{
    Base2DPassDef, DrawBase2D, DrawBase2DDesc, DrawBase2DTransparent, DrawBase2DTransparentDesc,
};
use crate::pod::SpriteArgs;
use crate::resources::Tint;
use crate::{SpriteRender, SpriteSheet, Texture};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::Transform;
use rendy::shader::SpirvShader;

/// Implementation of `Base3DPassDef` describing a simple shaded 3D pass.
#[derive(Debug)]
pub struct Flat2DPassDef;
impl Base2DPassDef for Flat2DPassDef {
    const NAME: &'static str = "Flat 2D";
    type SpriteComponent = SpriteRender;
    type SpriteData = SpriteArgs;
    fn vertex_shader() -> &'static SpirvShader {
        &super::SPRITE_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &super::SPRITE_FRAGMENT
    }

    fn get_args<'a>(
        tex_storage: &AssetStorage<Texture>,
        sprite_storage: &'a AssetStorage<SpriteSheet>,
        sprite_component: &Self::SpriteComponent,
        transform: &Transform,
        tint: Option<&Tint>,
    ) -> Option<(Self::SpriteData, &'a [Handle<Texture>])> {
        SpriteArgs::from_data(
            tex_storage,
            sprite_storage,
            sprite_component,
            transform,
            tint,
        )
        .map(|(data, texture)| (data, std::slice::from_ref(texture)))
    }
}

/// Describes a simple shaded 3D pass.
pub type DrawFlat2DDesc<B> = DrawBase2DDesc<B, Flat2DPassDef>;
/// Draws a simple shaded 3D pass.
pub type DrawFlat2D<B> = DrawBase2D<B, Flat2DPassDef>;

/// Describes a simple shaded 3D pass with transparency
pub type DrawFlat2DTransparentDesc<B> = DrawBase2DTransparentDesc<B, Flat2DPassDef>;
/// Draws a simple shaded 3D pass with transparency
pub type DrawFlat2DTransparent<B> = DrawBase2DTransparent<B, Flat2DPassDef>;
