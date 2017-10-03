//! Rendering system.

use ecs::{Fetch, FetchMut, System};
use ecs::rendering::resources::{Factory, WindowMessages};
use renderer::Renderer;
use renderer::pipe::{PipelineData, PolyPipeline};

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer) -> Self {
        Self { pipe, renderer }
    }
}

impl<'a, P> System<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    type SystemData = (
        Fetch<'a, Factory>,
        FetchMut<'a, WindowMessages>,
        <P as PipelineData<'a>>::Data,
    );

    fn run(&mut self, (factory, mut window_messages, data): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        use std::time::Duration;

        for mut command in window_messages.queue.drain() {
            command(self.renderer.window());
        }

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.renderer
            .draw(&mut self.pipe, data, Duration::from_secs(0));
    }
}
