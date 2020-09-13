use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{ecs::prelude::*, Transform};
use amethyst_error::Error;
use amethyst_rendy::{
    bundle::{RenderOrder, RenderPlan, Target},
    pass::{Base2DPassDef, DrawBase2DDesc, DrawBase2DTransparentDesc},
    pod::{SpriteArgs, ViewArgs},
    rendy::{graph::render::RenderGroupDesc, hal::pso::ShaderStageFlags, shader::SpirvShader},
    resources::Tint,
    submodules::gather::CameraGatherer,
    Backend, Factory, RenderPlugin, SpriteRender, SpriteSheet, Texture,
};

use glsl_layout::AsStd140;

//Load Shaders
lazy_static::lazy_static! {
    // These uses the precompiled shaders.
    // These can be obtained using glslc.exe in the vulkan sdk.
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/vertex/color_replacement.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/fragment/color_replacement.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

pub struct ColorReplacementSprite {
    pub sprite: SpriteRender,
}

impl Component for ColorReplacementSprite {
    type Storage = DenseVecStorage<Self>;
}

/// Implementation of `Base2DPassDef` describing the Color Replacement Pass.
#[derive(Debug)]
pub struct ColorReplacementPassDef;
impl Base2DPassDef for ColorReplacementPassDef {
    const NAME: &'static str = "Flat 2D";
    type SpriteComponent = ColorReplacementSprite;
    type SpriteData = SpriteArgs;
    type UniformType = ViewArgs;

    const TEXTURE_COUNT: usize = 1;

    fn vertex_shader() -> &'static SpirvShader {
        &VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &FRAGMENT
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
            &sprite_component.sprite,
            transform,
            tint,
        )
        .map(|(data, texture)| (data, vec![texture.clone()]))
    }

    fn get_uniform(world: &World) -> <ViewArgs as AsStd140>::Std140 {
        CameraGatherer::gather(world).projview
    }
}

/// Describes a Color Replacement Pass.
pub type ColorReplacementPassDesc<B> = DrawBase2DDesc<B, ColorReplacementPassDef>;

/// Describes a Color Replacement Pass with transparency
pub type ColorReplacementPassTransparentDesc<B> =
    DrawBase2DTransparentDesc<B, ColorReplacementPassDef>;

/// A [RenderPlugin] for drawing 2d objects with Color Replacement.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderColorReplacement {
    target: Target,
}

impl<B: Backend> RenderPlugin<B> for RenderColorReplacement {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        world.register::<ColorReplacementSprite>();
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(
                RenderOrder::Opaque,
                ColorReplacementPassDesc::new().builder(),
            )?;
            ctx.add(
                RenderOrder::Transparent,
                ColorReplacementPassTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}
