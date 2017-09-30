//! Rendering system.

use ecs::{Fetch, FetchMut, System};
use ecs::rendering::events::WindowModifierEvent;
use ecs::rendering::resources::Factory;
use renderer::Renderer;
use renderer::pipe::{PipelineData, PolyPipeline};
use shrev::{EventHandler, EventReadData, ReaderId};

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem<P> {
    pipe: P,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    reader_id: ReaderId,
}

impl<P> RenderSystem<P>
where
    P: PolyPipeline,
{
    /// Create a new render system
    pub fn new(pipe: P, renderer: Renderer, reader_id: ReaderId) -> Self {
        Self {
            pipe,
            renderer,
            reader_id,
        }
    }
}

impl<'a, P> System<'a> for RenderSystem<P>
where
    P: PolyPipeline,
{
    type SystemData = (
        Fetch<'a, Factory>,
        <P as PipelineData<'a>>::Data,
        FetchMut<'a, EventHandler<WindowModifierEvent>>,
    );

    fn run(&mut self, (factory, data, events): Self::SystemData) {
        //Read all window-modifying events
        match events.read(&mut self.reader_id) {
            Ok(EventReadData::Data(data)) => {
                let mut win = self.renderer.window_mut();
                for ev in data.collect::<Vec<_>>() {
                    let closure = ev.modify;
                    closure(win);
                }
            }
            _ => {}
        }

        #[cfg(feature = "profiler")]
        profile_scope!("render_system");
        use std::time::Duration;

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.renderer
            .draw(&mut self.pipe, data, Duration::from_secs(0));
    }
}
