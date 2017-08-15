//! The core engine framework.

use config::Config;
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use ecs::resources::input::InputHandler;
use ecs::components::{LocalTransform, Transform, Child, Init};
use ecs::systems::RenderSystem;
use engine::Engine;
use error::{Error, Result};
use event::Event;
use rayon::ThreadPool;
use state::{State, StateMachine};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use timing::{Stopwatch, Time};
use winit::EventsLoop;
use display_config::DisplayConfig;

#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};

/// User-friendly facade for building games. Manages main loop.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Application<'a, 'b> {
    #[derivative(Debug = "ignore")]
    dispatcher: Dispatcher<'a, 'b>,
    #[derivative(Debug = "ignore")]
    events: EventsLoop,
    #[derivative(Debug = "ignore")]
    engine: Engine,
    states: StateMachine<'a>,
    time: Time,
    timer: Stopwatch,
}

impl<'a, 'b> Application<'a, 'b> {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 'a>(initial_state: S) -> Result<Application<'a, 'b>> {
        ApplicationBuilder::new(initial_state).build()
    }

    /// Builds a new Application with the given settings.
    pub fn build<S>(initial_state: S) -> ApplicationBuilder<'a, 'b, S>
        where S: State + 'a
    {
        ApplicationBuilder::new(initial_state)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.time.delta_time = self.timer.elapsed();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("initialize");

        self.engine.world.add_resource(self.time.clone());
        self.states.start(&mut self.engine);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        {
            let mut time = self.engine.world.write_resource::<Time>();
            time.delta_time = self.time.delta_time;
            time.fixed_step = self.time.fixed_step;
            time.last_fixed_update = self.time.last_fixed_update;
        }

        {
            let engine = &mut self.engine;
            let states = &mut self.states;
            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            self.events.poll_events(|event| {
                states.handle_event(engine, event);
            });
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("fixed_update");
            if self.time.last_fixed_update.elapsed() >= self.time.fixed_step {
                self.states.fixed_update(&mut self.engine);
                self.time.last_fixed_update += self.time.fixed_step;
            }

            #[cfg(feature = "profiler")]
            profile_scope!("update");
            self.states.update(&mut self.engine);
        }

        #[cfg(feature = "profiler")]
        profile_scope!("dispatch");
        self.dispatcher.dispatch(&mut self.engine.world.res);

        #[cfg(feature="profiler")]
        profile_scope!("maintain");
        self.engine.world.maintain();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder.
    }
}

#[cfg(feature = "profiler")]
impl<'a, 'b> Drop for Application<'a, 'b> {
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<'a, 'b, T: State + 'a> {
    base_path: PathBuf,
    // config: Config,
    disp_builder: DispatcherBuilder<'a, 'b>,
    errors: Vec<Error>,
    initial_state: T,
    world: World,
    /// Allows to create `RenderSystem`
    // TODO: Come up with something clever
    pub events: EventsLoop,
}

impl<'a, 'b, T: State + 'a> ApplicationBuilder<'a, 'b, T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T) -> Self {
        ApplicationBuilder {
            base_path: format!("{}/resources", env!("CARGO_MANIFEST_DIR")).into(),
            disp_builder: DispatcherBuilder::new(),
            errors: Vec::new(),
            initial_state: initial_state,
            world: World::new(),
            events: EventsLoop::new(),
        }
    }

    /// Registers a given component type.
    pub fn register<C: Component>(mut self) -> Self {
        self.world.register::<C>();
        self
    }

    /// Inserts a barrier which assures that all systems added before the
    /// barrier are executed before the ones after this barrier.
    ///
    /// Does nothing if there were no systems added since the last call to
    /// `add_barrier()`. Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    pub fn add_barrier(mut self) -> Self {
        self.disp_builder = self.disp_builder.add_barrier();
        self
    }

    /// Adds a given system `sys`, assigns it the string identifier `name`,
    /// and marks it dependent on systems `dep`.
    /// Note: all dependencies should be added before you add depending system
    pub fn with<S>(mut self, sys: S, name: &str, dep: &[&str]) -> Self
        where for<'c> S: System<'c> + Send + 'a + 'b
    {
        self.disp_builder = self.disp_builder.add(sys, name, dep);
        self
    }

    /// Adds a given thread-local system `sys` to the Application.
    ///
    /// All thread-local systems are executed sequentially after all
    /// non-thread-local systems.
    pub fn with_thread_local<S>(mut self, sys: S) -> Self
        where for<'c> S: System<'c> + 'a + 'b
    {
        self.disp_builder = self.disp_builder.add_thread_local(sys);
        self
    }

    /// Builds the Application and returns the result.
    pub fn build(self) -> Result<Application<'a, 'b>> {
        use num_cpus;
        use rayon::Configuration;

        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        let num_cores = num_cpus::get();
        let cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(cfg).map(|p| Arc::new(p)).map_err(|_| Error::Application)?;

        Ok(Application {
            engine: Engine::new(&self.base_path, pool.clone(), self.world),
            // config: self.config,
            states: StateMachine::new(self.initial_state),
            events: self.events,
            dispatcher: self.disp_builder.with_pool(pool).build(),
            time: Time::default(),
            timer: Stopwatch::new(),
        })
    }
}
