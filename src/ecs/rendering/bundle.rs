//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;
use core::bundle::{ECSBundle, Result};

use assets::{AssetFuture, BoxedErr, Loader};
use ecs::{World, DispatcherBuilder};
use ecs::rendering::components::*;
use ecs::rendering::resources::{AmbientColor, Factory, ScreenDimensions, WindowMessages};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;


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

impl<'a, 'b, P> ECSBundle<'a, 'b> for RenderBundle<P>
where
    P: PipelineBuild + Clone,
    P::Pipeline: 'b,
{
    fn build(
        &self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        use specs::common::{Errors, Merge};

        let mut renderer = {
            let mut renderer = Renderer::build();

            if let Some(config) = self.display_config.to_owned() {
                renderer.with_config(config);
            }
            let renderer = renderer
                .build()
                .map_err(|err| BoxedErr::new(err))?;

            renderer
        };

        let pipe = renderer
            .create_pipe(self.pipe.clone())
            .map_err(|err| BoxedErr::new(err))?;

        use cgmath::Deg;
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        let (w, h) = renderer.window().get_inner_size_pixels().unwrap();

        world.add_resource(Factory::new());
        world.add_resource(cam);
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
        world.add_resource(WindowMessages::new());
        world.add_resource(ScreenDimensions::new(w, h));

        world.register::<Transform>();
        world.register::<LightComponent>();
        world.register::<MaterialComponent>();
        world.register::<MeshComponent>();
        world.register::<TextureComponent>();

        // FIXME: asset stuff, enable/disable flag for this?
        {
            let (mesh_context, texture_context) = {
                let factory = world.read_resource::<Factory>();
                (
                    MeshContext::new((&*factory).clone()),
                    TextureContext::new((&*factory).clone()),
                )
            };

            {
                let mut loader = world.write_resource::<Loader>();
                loader.register(mesh_context);
                loader.register(texture_context);
            }

            world.register::<AssetFuture<MeshComponent>>();
            world.register::<AssetFuture<TextureComponent>>();
            world.register::<AssetFuture<MaterialComponent>>();
            world.add_resource(Errors::new());

            builder = builder
                .add(Merge::<AssetFuture<MaterialComponent>>::new(), "", &[])
                .add(Merge::<AssetFuture<MeshComponent>>::new(), "", &[])
                .add(Merge::<AssetFuture<TextureComponent>>::new(), "", &[]);
        }

        builder = builder.add_thread_local(RenderSystem::new(pipe, renderer));

        Ok(builder)
    }
}
