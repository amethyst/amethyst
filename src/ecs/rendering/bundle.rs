//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;

use app::ApplicationBuilder;
use assets::{BoxedErr};
use ecs::ECSBundle;
use ecs::rendering::resources::{AmbientColor, Factory, ScreenDimensions, WindowMessages};
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
        let mut renderer = {
            let mut renderer = Renderer::build();

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
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        let (w, h) = renderer.window().get_inner_size_pixels().unwrap();

        builder = builder
            .with_resource(Factory::new())
            .with_resource(cam)
            .with_resource(AmbientColor(Rgba::from([0.01; 3])))
            .with_resource(WindowMessages::new())
            .with_resource(ScreenDimensions::new(w, h))
            .register::<Transform>()
            .register::<Light>()
            .register::<Material>()
            .register::<Mesh>()
            .register::<Texture>();

        // TODO: register assets with loader, eventually.

        builder = builder.with_thread_local(RenderSystem::new(pipe, renderer));

        Ok(builder)
    }
}
