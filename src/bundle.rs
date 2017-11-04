use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use assets::Loader;
use core::{ECSBundle, Result, Stopwatch, Time};
use core::frame_limiter::FrameLimiter;
use ecs::{DispatcherBuilder, World};
use ecs::common::Errors;
use rayon::{Configuration, ThreadPool};
use renderer::Event;
use shrev::EventChannel;
#[cfg(feature = "profiler")]
use thread_profiler::register_thread_with_profiler;

pub struct AppBundle {
    path: PathBuf,
}

impl AppBundle {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }
}

impl<'a, 'b> ECSBundle<'a, 'b> for AppBundle {
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        let cfg = Configuration::new();
        #[cfg(feature = "profiler")]
        let cfg = cfg.start_handler(|index| {
            register_thread_with_profiler(format!("thread_pool{}", index));
        });
        let pool = ThreadPool::new(cfg)
            .map(|p| Arc::new(p))
            .map_err(|err| err.description().to_string())?;
        world.add_resource(Loader::new(self.path, pool.clone()));
        world.add_resource(EventChannel::<Event>::with_capacity(2000));
        world.add_resource(Errors::new());
        world.add_resource(pool);
        world.add_resource(FrameLimiter::default());
        world.add_resource(Stopwatch::default());
        let mut time = Time::default();
        time.set_fixed_time(Duration::new(0, 16666666));
        world.add_resource(time);
        Ok(dispatcher)
    }
}
