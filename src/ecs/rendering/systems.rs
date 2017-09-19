//! Rendering system.

use assets::BoxedErr;
use ecs::{Fetch, System, World};
use ecs::rendering::resources::{Factory, AmbientColor};
use ecs::rendering::{LightComponent, MaterialComponent, MeshComponent};
use ecs::transform::components::*;
use error::{Error, Result};
//use renderer::prelude::*;
use renderer::{Config as DisplayConfig, Renderer, Rgba, Light, Mesh, Material};
use renderer::pipe::{PolyPipeline, PipelineData, PipelineBuild};

use winit::EventsLoop;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
}

impl<P> RenderSystem<P>
    where P: PolyPipeline,
{
    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer) -> Self {
        Self {
            pipe,
            renderer,
        }
    }
}

impl<'a, P> System<'a> for RenderSystem<P>
    where P: PolyPipeline,
{
    type SystemData = (
        Fetch<'a, Factory>,
        <P as PipelineData<'a>>::Data
    );

    fn run(
        &mut self,
        (factory, data): Self::SystemData,
    ) {
        use std::time::Duration;

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.renderer.draw(&mut self.pipe, data, Duration::from_secs(0));
    }
}
