use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::ecs::prelude::*;
use amethyst_core::math::{convert, Matrix4, Vector4};
use amethyst_core::{Time, Transform};
use amethyst_error::Error;
use amethyst_rendy::bundle::{RenderOrder, RenderPlan, Target};
use amethyst_rendy::pass::{
    Base2DPassDef, DrawBase2D, DrawBase2DDesc, DrawBase2DTransparent, DrawBase2DTransparentDesc,
};
use amethyst_rendy::pod::IntoPod;
use amethyst_rendy::pod::{SpriteArgs, ViewArgs};
use amethyst_rendy::rendy::graph::render::{PrepareResult, RenderGroup, RenderGroupDesc};
use amethyst_rendy::rendy::hal::pso::ShaderStageFlags;
use amethyst_rendy::rendy::shader::SpirvShader;
use amethyst_rendy::resources::Tint;
use amethyst_rendy::submodules::gather::CameraGatherer;
use amethyst_rendy::{
    ActiveCamera, Backend, Camera, Factory, RenderPlugin, SpriteRender, SpriteSheet, Texture,
};
use glsl_layout::{mat4, AsStd140};

///Load Shaders
lazy_static::lazy_static! {
    // These uses the precompiled shaders.
    // These can be obtained using glslc.exe in the vulkan sdk.
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/vertex/blend.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/fragment/blend.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

///Blend Sprite Definition
pub struct BlendSprite {
    pub sprites: [SpriteRender; 2],
}

impl Component for BlendSprite {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct ViewTimeArgs {
    /// Premultiplied Proj-View matrix
    pub proj_view: mat4,

    ///Time
    pub time: f32,
}

/// Implementation of `Base2DPassDef` describing a Blending Sprite pass.
#[derive(Debug)]
pub struct BlendSpritePassDef;
impl Base2DPassDef for BlendSpritePassDef {
    const NAME: &'static str = "Flat 2D";
    type SpriteComponent = BlendSprite;
    type SpriteData = SpriteArgs;
    type UniformType = ViewArgs;

    const TEXTURE_COUNT: usize = 2;

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
        let sprite_sheet1 = sprite_storage.get(&sprite_component.sprites[0].sprite_sheet)?;
        let sprite_sheet2 = sprite_storage.get(&sprite_component.sprites[1].sprite_sheet)?;
        if !tex_storage.contains(&sprite_sheet1.texture)
            || !tex_storage.contains(&sprite_sheet2.texture)
        {
            return None;
        }

        let sprite = &sprite_sheet1.sprites[sprite_component.sprites[0].sprite_number];

        let transform = convert::<_, Matrix4<f32>>(*transform.global_matrix());
        let dir_x = transform.column(0) * sprite.width;
        let dir_y = transform.column(1) * -sprite.height;
        let pos = transform * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

        Some((
            SpriteArgs {
                dir_x: dir_x.xy().into_pod(),
                dir_y: dir_y.xy().into_pod(),
                pos: pos.xy().into_pod(),
                u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
                v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
                depth: pos.z,
                tint: tint.map_or([1.0; 4].into(), |t| {
                    // Shaders expect linear RGBA; convert sRGBA to linear RGBA
                    let (r, g, b, a) = t.0.into_linear().into_components();
                    [r, g, b, a].into()
                }),
            },
            vec![sprite_sheet1.texture.clone(), sprite_sheet2.texture.clone()],
        ))
    }

    fn get_uniform(world: &World) -> <ViewArgs as AsStd140>::Std140 {
        CameraGatherer::gather(world).projview
    }
}

/// Describes a simple flat 2D pass.
pub type BlendSpritePassDesc<B> = DrawBase2DDesc<B, BlendSpritePassDef>;
/// Draws a simple flat 2D pass.
pub type BlendSpritePass<B> = DrawBase2D<B, BlendSpritePassDef>;

/// Describes a simple flat 2D pass with transparency
pub type BlendSpritePassTransparentDesc<B> = DrawBase2DTransparentDesc<B, BlendSpritePassDef>;
/// Draws a simple flat 2D pass with transparency
pub type BlendSpritePassTransparent<B> = DrawBase2DTransparent<B, BlendSpritePassDef>;

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderBlendSprite {
    target: Target,
}

impl RenderBlendSprite {
    /// Set target to which 2d sprites will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderBlendSprite {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        world.register::<BlendSprite>();
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, BlendSpritePassDesc::new().builder())?;
            ctx.add(
                RenderOrder::Transparent,
                BlendSpritePassTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}
