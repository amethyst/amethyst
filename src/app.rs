//! The core engine framework.

use std::sync::Arc;

use rayon::ThreadPool;
use renderer::PipelineBuilder;
use shred::Resource;
use winit::EventsLoop;

use assets::{Asset, Loader, Store};
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use engine::Engine;
use error::{Error, Result};
use state::{State, StateMachine};
use timing::{Stopwatch, Time};
use renderer::Config as DisplayConfig;

#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};

/// User-friendly facade for building games. Manages main loop.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Application<'a, 'b> {
    /// The `engine` struct, holding world and thread pool.
    #[derivative(Debug = "ignore")]
    pub engine: Engine,

    #[derivative(Debug = "ignore")]
    dispatcher: Dispatcher<'a, 'b>,
    #[derivative(Debug = "ignore")]
    events: EventsLoop,
    states: StateMachine<'a>,
    time: Time,
    timer: Stopwatch,
}

impl<'a, 'b> Application<'a, 'b> {
    /// Creates a new Application with the given initial game state.
    pub fn new<S: State + 'a>(initial_state: S) -> Result<Application<'a, 'b>> {
        ApplicationBuilder::new(initial_state)?.build()
    }

    /// Builds a new Application with the given settings.
    pub fn build<S>(initial_state: S) -> Result<ApplicationBuilder<'a, 'b, S>>
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
    // config: Config,
    disp_builder: DispatcherBuilder<'a, 'b>,
    initial_state: T,
    world: World,
    pool: Arc<ThreadPool>,
    /// Allows to create `RenderSystem`
    // TODO: Come up with something clever
    pub events: EventsLoop,
}

impl<'a, 'b, T: State + 'a> ApplicationBuilder<'a, 'b, T> {
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T) -> Result<Self> {
        use num_cpus;
        use rayon::Configuration;

        let num_cores = num_cpus::get();
        let cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(cfg).map(|p| Arc::new(p)).map_err(|_| Error::Application)?;
        let mut world = World::new();
        let base_path = format!("{}/resources", env!("CARGO_MANIFEST_DIR"));
        world.add_resource(Loader::new(base_path, pool.clone()));

        Ok(ApplicationBuilder {
            disp_builder: DispatcherBuilder::new(),
            initial_state: initial_state,
            world: world,
            events: EventsLoop::new(),
            pool: pool,
        })
    }

    /// Registers a given component type.
    pub fn register<C: Component>(mut self) -> Self {
        self.world.register::<C>();
        self
    }

    /// Adds an ECS resource which can be accessed from systems.
    pub fn add_resource<R>(mut self, res: R) -> Self
        where R: Resource
    {
        self.world.add_resource(res);

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
    /// If you want to register systems which can not be specified as dependencies,
    /// you can use "" as their name, which will not panic (using another name twice will).
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

    /// Automatically registers components, adds resources and the rendering system.
    pub fn with_renderer(mut self, pipe: PipelineBuilder, config: Option<DisplayConfig>) -> Result<Self> {
        use ecs::systems::{RenderSystem, SystemExt};
        let render_sys = RenderSystem::build((&self.events, pipe, config), &mut self.world)?;
        self = self.with_thread_local(render_sys);

        Ok(self
            .register_mesh_asset()
            .register_texture_asset()
            .register_material_not_yet_asset())


    }

    /// Add asset loader to resources
    pub fn add_store<I, S>(self, name: I, store: S) -> Self
        where I: Into<String>,
              S: Store + Send + Sync + 'static,
    {
        {
            let mut loader = self.world.write_resource::<Loader>();
            loader.add_store(name, store);
        }
        self
    }

    /// Register new context within the loader
    pub fn register_asset<A, F>(mut self, make_context: F) -> Self
        where A: Component + Asset + Clone + Send + Sync + 'static,
              F: FnOnce(&mut World) -> A::Context,
    {
        use assets::AssetFuture;
        use specs::common::{Merge, Errors};

        self.world.register::<A>();
        self.world.register::<AssetFuture<A>>();
        self.world.add_resource(Errors::new());
        self = self.with(Merge::<AssetFuture<A>>::new(), "", &[]);
        {
            let context = make_context(&mut self.world);
            let mut loader = self.world.write_resource::<Loader>();
            loader.register(context);
        }
        self
    }

    /// Builds the Application and returns the result.
    pub fn build(self) -> Result<Application<'a, 'b>> {

        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        Ok(Application {
            engine: Engine::new(self.pool.clone(), self.world),
            // config: self.config,
            states: StateMachine::new(self.initial_state),
            events: self.events,
            dispatcher: self.disp_builder.with_pool(self.pool).build(),
            time: Time::default(),
            timer: Stopwatch::new(),
        })
    }




    /// Register new context within the loader
    fn register_mesh_asset(self) -> Self {
        use ecs::components::*;
        use ecs::resources::Factory;
        self.register_asset::<MeshComponent, _>(|world| {
            let factory = world.read_resource::<Factory>();
            MeshContext::new((&*factory).clone())
        })
    }

    /// Register new context within the loader
    fn register_material_not_yet_asset(mut self) -> Self {
        use assets::AssetFuture;
        use ecs::components::*;
        use specs::common::Merge;

        self.world.register::<MaterialComponent>();
        self.world.register::<AssetFuture<MaterialComponent>>();
        self = self.with(Merge::<AssetFuture<MaterialComponent>>::new(), "", &[]);
        self
    }

    /// Register new context within the loader
    fn register_texture_asset(self) -> Self {
        use ecs::components::*;
        use ecs::resources::Factory;
        self.register_asset::<TextureComponent, _>(|world| {
            let factory = world.read_resource::<Factory>();
            TextureContext::new((&*factory).clone())
        })
    }
}
