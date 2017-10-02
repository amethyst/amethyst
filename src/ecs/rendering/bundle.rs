//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;

use app::ApplicationBuilder;
use assets::{AssetFuture, BoxedErr, Loader};
use ecs::ECSBundle;
use ecs::rendering::components::*;
use ecs::rendering::resources::{AmbientColor, Factory};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;
use error::{Error, Result};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
/// Will add `RenderSystem` as a thread local system.
///
/// ## Errors
///
/// Returns errors related to:
///
/// * Renderer creation
/// * Pipeline creation
///
pub struct RenderBundle<P> {
    pipe: P,
    display_config: Option<DisplayConfig>,
}

impl<P> RenderBundle<P>
where
    P: PipelineBuild,
{
    /// Create a new render bundle with the given pipeline
    pub fn new(pipe: P) -> Self {
        Self {
            pipe,
            display_config: None,
        }
    }

    /// Use the given display config for configuring window and render properties
    pub fn with_config(mut self, config: DisplayConfig) -> Self {
        self.display_config = Some(config);
        self
    }
}

impl<'a, 'b, T, P> ECSBundle<'a, 'b, T> for RenderBundle<P>
where
    P: PipelineBuild + Clone,
    P::Pipeline: 'b,
{
    fn build(
        &self,
        mut builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        use specs::common::{Errors, Merge};

        let mut renderer = {
            let mut renderer = Renderer::build(&builder.events);

            if let Some(config) = self.display_config.to_owned() {
                renderer.with_config(config);
            }
            let renderer = renderer
                .build()
                .map_err(|err| Error::System(BoxedErr::new(err)))?;

            renderer
        };

        let pipe = renderer
            .create_pipe(self.pipe.clone())
            .map_err(|err| Error::System(BoxedErr::new(err)))?;

        use cgmath::Deg;
        use renderer::{Camera, LocalTransform, Projection};

        builder = builder
            .with_resource(Factory::new())
            .with_resource(AmbientColor(Rgba::from([0.01; 3])))
            .register::<Camera>()
            .register::<Transform>()
            .register::<LightComponent>()
            .register::<MaterialComponent>()
            .register::<MeshComponent>()
            .register::<TextureComponent>();

        let cam = Camera::new(Projection::perspective(1.3, Deg(60.0)).into());
        let mut cam_trans = LocalTransform::default();

        cam_trans.translation = [0.0, 0.0, -4.0].into();
        cam_trans.look_at([0.0, 0.0, 0.0].into());

        builder
            .world
            .create_entity()
            .with(cam)
            .with(cam_trans)
            .build()
        ;

        // FIXME: asset stuff, enable/disable flag for this?
        {
            let (mesh_context, texture_context) = {
                let factory = builder.world.read_resource::<Factory>();
                (
                    MeshContext::new((&*factory).clone()),
                    TextureContext::new((&*factory).clone()),
                )
            };

            {
                let mut loader = builder.world.write_resource::<Loader>();
                loader.register(mesh_context);
                loader.register(texture_context);
            }

            builder = builder
                .register::<AssetFuture<MeshComponent>>()
                .register::<AssetFuture<TextureComponent>>()
                .register::<AssetFuture<MaterialComponent>>()
                .with_resource(Errors::new())
                .with(Merge::<AssetFuture<MaterialComponent>>::new(), "", &[])
                .with(Merge::<AssetFuture<MeshComponent>>::new(), "", &[])
                .with(Merge::<AssetFuture<TextureComponent>>::new(), "", &[]);
        }

        builder = builder.with_thread_local(RenderSystem::new(pipe, renderer));

        Ok(builder)
    }
}
