use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{ecs::prelude::*, Time, Transform};
use amethyst_error::Error;
use amethyst_rendy::{
    bundle::{RenderOrder, RenderPlan, Target},
    pass::{Base2DPassDef, DrawBase2DDesc, DrawBase2DTransparentDesc},
    pod::SpriteArgs,
    rendy::{graph::render::RenderGroupDesc, hal::pso::ShaderStageFlags, shader::SpirvShader},
    resources::Tint,
    ActiveCamera, Backend, Camera, Factory, RenderPlugin, SpriteRender, SpriteSheet, Texture,
};

use glsl_layout::{mat4, AsStd140};

//Load Shaders
lazy_static::lazy_static! {
    // These uses the precompiled shaders.
    // These can be obtained using glslc.exe in the vulkan sdk.
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/vertex/fire.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("./assets/shaders/compiled/fragment/fire.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

pub struct FireSprite {
    pub sprite: SpriteRender,
}

impl Component for FireSprite {
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

/// Implementation of `Base2DPassDef` describing a fire sprite pass.
#[derive(Debug)]
pub struct FireSpritePassDef;
impl Base2DPassDef for FireSpritePassDef {
    const NAME: &'static str = "Flat 2D";
    type SpriteComponent = FireSprite;
    type SpriteData = SpriteArgs;
    type UniformType = ViewTimeArgs;

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

    fn get_uniform(world: &World) -> <ViewTimeArgs as AsStd140>::Std140 {
        #[cfg(feature = "profiler")]
        profile_scope!("gather_cameras");

        let (active_camera, cameras, transforms) = <(
            Read<'_, ActiveCamera>,
            ReadStorage<'_, Camera>,
            ReadStorage<'_, Transform>,
        )>::fetch(world);

        let defcam = Camera::standard_2d(1.0, 1.0);
        let identity = Transform::default();

        let (camera, transform) = active_camera
            .entity
            .as_ref()
            .and_then(|ac| {
                cameras
                    .get(*ac)
                    .map(|camera| (camera, transforms.get(*ac).unwrap_or(&identity)))
            })
            .unwrap_or_else(|| {
                (&cameras, &transforms)
                    .join()
                    .next()
                    .unwrap_or((&defcam, &identity))
            });

        let proj = &camera.matrix;
        let view = transform.global_view_matrix();

        let proj_view: [[f32; 4]; 4] = ((*proj) * view).into();

        let time = world.fetch::<Time>().absolute_real_time_seconds() as f32;

        ViewTimeArgs {
            proj_view: proj_view.into(),
            time,
        }
        .std140()
    }
}

/// Describes a fire sprite pass.
pub type FireSpritePassDesc<B> = DrawBase2DDesc<B, FireSpritePassDef>;

/// Describes a fire sprite pass with transparency
pub type FireSpritePassTransparentDesc<B> = DrawBase2DTransparentDesc<B, FireSpritePassDef>;

/// A [RenderPlugin] for drawing 2d  fire sprite pass.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderFireSprite {
    target: Target,
}

impl<B: Backend> RenderPlugin<B> for RenderFireSprite {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        world.register::<FireSprite>();
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, FireSpritePassDesc::new().builder())?;
            ctx.add(
                RenderOrder::Transparent,
                FireSpritePassTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}
