//! ECS rendering bundle

use renderer::Config as DisplayConfig;
use renderer::Rgba;
use renderer::pipe::PipelineBuild;
use renderer::prelude::*;

use app::ApplicationBuilder;
use assets::BoxedErr;
use ecs::ECSBundle;
use ecs::rendering::resources::{AmbientColor, ScreenDimensions, WindowMessages};
use ecs::rendering::systems::RenderSystem;
use ecs::transform::components::*;
use error::{Error, Result};

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
pub struct RenderBundle;

impl RenderBundle {
    /// Create a new render bundle
    pub fn new() -> Self {
        Self {}
    }
}

/// Create render system
pub fn create_render_system<P>(
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

impl<'a, 'b, T> ECSBundle<'a, 'b, T> for RenderBundle {
    fn build(
        &self,
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
            .register::<Mesh>()
            .register::<Texture>()
            .register::<Handle<Mesh>>()
            .register::<Handle<Texture>>()
            .with_resource(AssetStorage::<Mesh>::new())
            .with_resource(AssetStorage::<Texture>::new());

        // TODO: register assets with loader, eventually.

        Ok(builder)
    }
}
