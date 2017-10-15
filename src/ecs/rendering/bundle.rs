//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::formats::TextureData;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;

use app::ApplicationBuilder;
use assets::{BoxedErr, Loader};
use ecs::{ECSBundle, World};
use ecs::rendering::resources::{AmbientColor, ScreenDimensions, WindowMessages};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;
use error::{Error, Result};
use renderer::MaterialDefaults;

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
pub struct RenderBundle<P> {
    config: Option<DisplayConfig>,
    pipe: P,
}

impl<P: PipelineBuild + Clone> RenderBundle<P> {
    /// Create a new render bundle
    pub fn new(pipe: P, config: Option<DisplayConfig>) -> Self {
        RenderBundle { config, pipe }
    }
}

/// Create render system
fn create_render_system<P>(
    pipe: P,
    display_config: Option<DisplayConfig>,
) -> Result<RenderSystem<P::Pipeline>>
where
    P: PipelineBuild + Clone,
{
    let mut renderer = {
        let mut renderer = Renderer::build();

        if let Some(config) = display_config.to_owned() {
            renderer.with_config(config);
        }
        let renderer = renderer
            .build()
            .map_err(|err| Error::System(BoxedErr::new(err)))?;

        renderer
    };

    let pipe = renderer
        .create_pipe(pipe.clone())
        .map_err(|err| Error::System(BoxedErr::new(err)))?;

    Ok(RenderSystem::new(pipe, renderer))
}

impl<'a, 'b, P, T> ECSBundle<'a, 'b, T> for RenderBundle<P>
where
    P: PipelineBuild + Clone,
    <P as PipelineBuild>::Pipeline: 'b,
{
    fn build(
        self,
        mut builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        use assets::{AssetStorage, Handle};
        use cgmath::Deg;
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        builder = builder
            .with_resource(cam)
            .with_resource(AmbientColor(Rgba::from([0.01; 3])))
            .with_resource(WindowMessages::new())
            .with_resource(ScreenDimensions::new(100, 100))
            .register::<Transform>()
            .register::<Light>()
            .register::<Material>()
            .register::<Handle<Mesh>>()
            .register::<Handle<Texture>>()
            .with_resource(AssetStorage::<Mesh>::new())
            .with_resource(AssetStorage::<Texture>::new())
            .with_local(create_render_system(self.pipe, self.config)?);

        let mat = create_default_mat(&builder.world);
        builder = builder.with_resource(MaterialDefaults(mat));

        // TODO: register assets with loader, eventually.

        Ok(builder)
    }
}

fn create_default_mat(world: &World) -> Material {
    let loader = world.read_resource::<Loader>();

    let albedo = TextureData::color([0.5, 0.5, 0.5, 1.0]);
    let emission = TextureData::color([0.0; 4]);
    let normal = TextureData::color([0.5, 0.5, 1.0, 1.0]);
    let metallic = TextureData::color([0.0; 4]);
    let roughness = TextureData::color([0.5; 4]);
    let ambient_occlusion = TextureData::color([1.0; 4]);
    let caveat = TextureData::color([1.0; 4]);

    let tex_storage = world.read_resource();

    let albedo = loader.load_from_data(albedo, &tex_storage);
    let emission = loader.load_from_data(emission, &tex_storage);
    let normal = loader.load_from_data(normal, &tex_storage);
    let metallic = loader.load_from_data(metallic, &tex_storage);
    let roughness = loader.load_from_data(roughness, &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, &tex_storage);
    let caveat = loader.load_from_data(caveat, &tex_storage);

    Material {
        albedo,
        emission,
        normal,
        metallic,
        roughness,
        ambient_occlusion,
        caveat,
    }
}
