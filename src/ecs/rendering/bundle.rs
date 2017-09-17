//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::prelude::*;

use winit::EventsLoop;

use assets::{BoxedErr, Loader, AssetFuture};
use ecs::{World, DispatcherBuilder};
use ecs::ECSBundle;
use ecs::rendering::components::*;
use ecs::rendering::resources::{Factory, AmbientColor};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;
use error::{Error, Result};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset Loader, and add systems for merging
/// AssetFutures into their respective component.
///
/// Will add RenderSystem as a thread local system.
///
pub struct RenderBundle {
    pipe: PipelineBuilder,
    display_config: Option<DisplayConfig>,
}

impl RenderBundle {
    /// Create a new render bundle with the given pipeline
    pub fn new(pipe: PipelineBuilder) -> Self {
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

impl<'a, 'b, 'c> ECSBundle<'a, 'b, (&'c EventsLoop)> for RenderBundle {
    fn build(
        &self,
        events: (&'c EventsLoop),
        world: &mut World,
        mut dispatcher: DispatcherBuilder<'a, 'b>,
        _: &str,
        _: &[&str],
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        use specs::common::{Merge, Errors};

        let mut renderer = Renderer::build(events);

        if let Some(config) = self.display_config.to_owned() {
            renderer.with_config(config);
        }
        let mut renderer = renderer.build().map_err(
            |err| Error::System(BoxedErr::new(err)),
        )?;
        let pipe = renderer.create_pipe(self.pipe.to_owned()).map_err(|err| {
            Error::System(BoxedErr::new(err))
        })?;

        use cgmath::Deg;
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        world.add_resource(Factory::new());
        world.add_resource(cam);
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
        world.register::<Transform>();
        world.register::<LightComponent>();
        world.register::<MaterialComponent>();
        world.register::<MeshComponent>();
        world.register::<TextureComponent>();

        world.register::<AssetFuture<MeshComponent>>();
        world.register::<AssetFuture<TextureComponent>>();
        world.register::<AssetFuture<MaterialComponent>>();
        world.add_resource(Errors::new());

        // asset stuff, enable/disable flag for this?
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

            dispatcher = dispatcher.add(Merge::<AssetFuture<MaterialComponent>>::new(), "", &[]);
            dispatcher = dispatcher.add(Merge::<AssetFuture<MeshComponent>>::new(), "", &[]);
            dispatcher = dispatcher.add(Merge::<AssetFuture<TextureComponent>>::new(), "", &[]);
        }

        dispatcher =
            dispatcher.add_thread_local(RenderSystem::new(pipe, renderer, Scene::default()));

        Ok(dispatcher)
    }
}
