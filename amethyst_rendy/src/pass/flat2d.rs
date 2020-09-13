use crate::pass::{
    Base2DPassDef, DrawBase2D, DrawBase2DDesc, DrawBase2DTransparent, DrawBase2DTransparentDesc,
};
use crate::pod::{SpriteArgs, ViewArgs};
use crate::resources::Tint;
use crate::submodules::gather::CameraGatherer;
use crate::{SpriteRender, SpriteSheet, Texture};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::ecs::World;
use amethyst_core::Transform;
use glsl_layout::AsStd140;
use rendy::shader::SpirvShader;

/// Implementation of `Base2DPassDef` describing a simple flat 2D pass.
#[derive(Debug)]
pub struct Flat2DPassDef;
impl Base2DPassDef for Flat2DPassDef {
    const NAME: &'static str = "Flat 2D";
    const TEXTURE_COUNT: usize = 1;

    type SpriteComponent = SpriteRender;
    type SpriteData = SpriteArgs;
    type UniformType = ViewArgs;

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
    ) -> Option<(Self::SpriteData, Vec<Handle<Texture>>)> {
        SpriteArgs::from_data(
            tex_storage,
            sprite_storage,
            sprite_component,
            transform,
            tint,
        )
        .map(|(data, texture)| (data, vec![texture.clone()]))
    }

    fn get_uniform(world: &World) -> <ViewArgs as AsStd140>::Std140 {
        CameraGatherer::gather(world).projview
    }
}

/// Describes a simple flat 2D pass.
pub type DrawFlat2DDesc<B> = DrawBase2DDesc<B, Flat2DPassDef>;
/// Draws a simple flat 2D pass.
pub type DrawFlat2D<B> = DrawBase2D<B, Flat2DPassDef>;

/// Describes a simple flat 2D pass with transparency
pub type DrawFlat2DTransparentDesc<B> = DrawBase2DTransparentDesc<B, Flat2DPassDef>;
/// Draws a simple flat 2D pass with transparency
pub type DrawFlat2DTranspaerent<B> = DrawBase2DTransparent<B, Flat2DPassDef>;
