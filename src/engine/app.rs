//! The core engine framework.

use asset_manager::AssetManager;
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use ecs::components::{LocalTransform, Transform, Child, Init};
use ecs::resources::Time;
use engine::state::{State, StateMachine};
use engine::timing::Stopwatch;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rayon::{Configuration, ThreadPool};

#[cfg(feature="profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};

/// User-friendly facade for building games. Manages main loop.
pub struct Application<'a, 'b> {
    // Graphics and asset management structs.
    assets: AssetManager,
    dispatcher: Dispatcher<'a, 'b>,
    world: World,

    // State management and game loop timing structs.
    delta_time: Duration,
    fixed_step: Duration,
    last_fixed_update: Instant,
    states: StateMachine,
    timer: Stopwatch,
}

impl<'a, 'b> Application<'a, 'b> {
    /// Creates a new Application with the given initial game state, dispatcher and world,
    /// and display configuration.
    pub fn new<T>(initial_state: T,
                  disp: Dispatcher<'a, 'b>,
                  mut world: World)
                  -> Application<'a, 'b>
        where T: State + 'static
    {
        #[cfg(feature="profiler")]
        register_thread_with_profiler("Main".into());

        let mut assets = AssetManager::new();
        // assets.add_loader::<gfx_types::Factory>(factory);

        {
            let time = Time {
                delta_time: Duration::new(0, 0),
                fixed_step: Duration::new(0, 16666666),
                last_fixed_update: Instant::now(),
            };

            // world.add_resource::<AmbientLight>(AmbientLight::default());
            world.add_resource::<Time>(time);
            world.register::<Child>();
            // world.register::<DirectionalLight>();
            world.register::<Init>();
            world.register::<LocalTransform>();
            // world.register::<PointLight>();
            // world.register::<Renderable>();
            // world.register::<Transform>();
        }

        Application {
            assets: assets,
            states: StateMachine::new(initial_state),
            dispatcher: dispatcher,
            world: world,
            timer: Stopwatch::new(),
            delta_time: Duration::new(0, 0),
            fixed_step: Duration::new(0, 16666666),
            last_fixed_update: Instant::now(),
        }
    }

    /// Builds a new application using builder pattern.
    pub fn build<T>(initial_state: T) -> ApplicationBuilder<'a, T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        {
            #[cfg(feature="profiler")]
            profile_scope!("initialize");
            self.initialize();
        }

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.delta_time = self.timer.elapsed();
        }

        {
            #[cfg(feature="profiler")]
            profile_scope!("shutdown");
            self.shutdown();
        }
        #[cfg(feature="profiler")]
        self.write_profile();
    }

    /// Direct access to `World`
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        let world = &mut self.world;
        let assets = &mut self.assets;
        self.states.start(world, assets);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        use ecs::resources::ScreenDimensions;
        {
            #[cfg(feature="profiler")]
            profile_scope!("handle_events");
            // let events = self.gfx_device.poll_events();
            let world = &mut self.world;
            let assets = &mut self.assets;

            // self.states.handle_events(events.as_ref(), world, assets, pipe);

            #[cfg(feature="profiler")]
            profile_scope!("fixed_update");
            if self.last_fixed_update.elapsed() >= self.fixed_step {
                self.states.fixed_update(world, assets);
                self.last_fixed_update += self.fixed_step;
            }

            #[cfg(feature="profiler")]
            profile_scope!("update");
            self.states.update(world, assets);
        }

        #[cfg(feature="profiler")]
        profile_scope!("dispatch");
        self.dispatcher.dispatch(&mut self.world.res);

        #[cfg(feature="profiler")]
        profile_scope!("render_world");
        {
            let world = &mut self.world;
            // if let Some((w, h)) = self.gfx_device.get_dimensions() {
            //     let mut dim = world.write_resource::<ScreenDimensions>();
            //     dim.update(w, h);
            // }

            {
                let mut time = world.write_resource::<Time>();
                time.delta_time = self.delta_time;
                time.fixed_step = self.fixed_step;
                time.last_fixed_update = self.last_fixed_update;
            }
        }

        self.world.maintain();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder.
    }

    #[cfg(feature="profiler")]
    /// Writes thread_profiler profile.
    fn write_profile(&self) {
        // TODO: Specify filename in config.
        let path = format!("{}/thread_profile.json", env!("CARGO_MANIFEST_DIR"));
        write_profile(path.as_str());
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<'a, 'b, T: State + 'static> {
    initial_state: T,
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
    world: World,
}

impl<'a, 'b, T> ApplicationBuilder<'a, 'b, T>
    where T: State + 'static
{
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T) -> ApplicationBuilder<'a, 'b, T> {
        use num_cpus;
        use rayon::Configuration;

        let pool = Arc::new(ThreadPool::new(Configuration::new().num_threads(num_cpus::get())).expect("Failed to create thread pool"));

        ApplicationBuilder {
            initial_state: initial_state,
            dispatcher_builder: DispatcherBuilder::new().with_pool(pool),
            world: World::new(),
        }
    }

    /// Registers a given component type.
    pub fn register<C>(mut self) -> ApplicationBuilder<'a, 'b, T>
        where C: Component
    {
        self.world.register::<C>();
        self
    }

    /// Inserts a barrier which assures that all systems added before the barrier are executed
    /// before the ones after this barrier.
    /// Does nothing if there were no systems added since the last call to add_barrier().
    /// Thread-local systems are not affected by barriers; they're always executed at the end.
    pub fn add_barrier(mut self) -> ApplicationBuilder<'a, 'b, T> {
        self.dispatcher_builder = self.dispatcher_builder.add_barrier();
        self
    }

    /// Adds a given system `sys`, assigns it the string identifier `name`,
    /// and marks it dependent on systems `dep`.
    /// Note: all dependencies should be added before you add depending system
    pub fn with<S>(mut self, sys: S, name: &str, dep: &[&str]) -> ApplicationBuilder<'a, 'b, T>
        where for<'c> S: System<'c> + Send + 'a + 'b
    {
        self.dispatcher_builder = self.dispatcher_builder.add(sys, name, dep);
        self
    }

    /// Adds a given thread-local system `sys`
    /// All thread-local systems are executed sequentially after all non-thread-local systems
    pub fn with_thread_local<S>(mut self, sys: S) -> ApplicationBuilder<'a, 'b, T>
        where for<'c> S: System<'c> + 'a + 'b
    {
        self.dispatcher_builder = self.dispatcher_builder.add_thread_local(sys);
        self
    }

    /// Builds the Application and returns the result.
    pub fn done(self) -> Application<'a, 'b> {
        Application::new(self.initial_state,
                         self.dispatcher_builder.build(),
                         self.world)
    }
}
