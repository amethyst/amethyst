//! The core engine framework.

use std::sync::Arc;

use input::InputHandler;
use rayon::ThreadPool;
use renderer::Config as DisplayConfig;
use renderer::PipelineBuilder;
use shred::{Resource, ResourceId};
#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};
use winit::{Event, EventsLoop};

use assets::{Asset, Loader, Store};
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use engine::Engine;
use error::{Error, Result};
use state::{State, StateMachine};
use timing::{Stopwatch, Time};

/// User-friendly facade for building games. Manages main loop.
///
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
    /// This will create and allocate all the needed resources for
    /// the event loop of the game engine. It is a shortcut for convenience
    /// if you need more control over how the engine is configured you should
    /// be using [build](struct.Application.html#method.build) instead.
    ///
    /// # Parameters
    /// - `initial_state`: The initial State handler of your game See
    ///   [State](trait.State.html) for more information on what this is.
    ///
    /// # Returns
    ///
    /// Returns a `Result` type wrapping the `Application` type. See
    /// [errors](struct.Application.html#errors) for a full list of
    /// possible errors that can happen in the creation of a Application object.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `State` trait. e.g. Your initial
    ///        game logic.
    ///
    /// # Lifetimes
    ///
    /// - `a`: The lifetime of the `State` objects.
    /// - `b`: This lifetime is inherited from `specs` and `shred`, it is
    ///        the minimum lifetime of the systems used by `Application`
    ///
    /// # Errors
    ///
    /// Application will return an error if the internal threadpool fails
    /// to initialize correctly because of systems resource limitations
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    ///
    /// struct NullState;
    ///
    /// impl State for NullState {}
    ///
    /// fn main() {
    ///     let mut game = Application::new(NullState).expect("Failed to initialize");
    ///     game.run();
    /// }
    /// ~~~
    pub fn new<S>(initial_state: S) -> Result<Application<'a, 'b>>
    where
        S: State + 'a,
    {
        ApplicationBuilder::new(initial_state)?.build()
    }


    /// Creates a new ApplicationBuilder with the given initial game state.
    ///
    /// This is identical in function to [ApplicationBuilder::new](struct.ApplicationBuilder.html#method.new).
    pub fn build<S>(initial_state: S) -> Result<ApplicationBuilder<'a, 'b, S>>
    where
        S: State + 'a,
    {
        ApplicationBuilder::new(initial_state)
    }

    /// Run the gameloop until the game state indicates that the game is no
    /// longer running. This is done via the `State` returning `Trans::Quit` or
    /// `Trans::Pop` on the last state in from the stack. See full
    /// documentation on this in [State](trait.State.html) documentation.
    ///
    /// # Examples
    ///
    /// See the example supplied in the
    /// [`new`](struct.Application.html#examples) method.
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
            if engine.world.res.has_value(
                ResourceId::new::<InputHandler>(),
            )
            {
                engine
                    .world
                    .write_resource::<InputHandler>()
                    .advance_frame();
            }
            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            self.events.poll_events(|event| {
                if engine.world.res.has_value(
                    ResourceId::new::<InputHandler>(),
                )
                {
                    let mut input = engine.world.write_resource::<InputHandler>();
                    if let Event::WindowEvent { ref event, .. } = event {
                        input.send_event(&event);
                    }
                }
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
    /// Creates a new [ApplicationBuilder](struct.ApplicationBuilder.html) instance
    /// that wraps the initial_state. This is the more verbose way of initializing
    /// your application if you require specific configuration details to be changed
    /// away from the default.
    ///
    /// # Parameters
    /// - `initial_state`: The initial State handler of your game. See
    ///   [State](trait.State.html) for more information on what this is.
    ///
    /// # Returns
    ///
    /// Returns a `Result` type wrapping the `Application` type. See
    /// [errors](struct.Application.html#errors) for a full list of
    /// possible errors that can happen in the creation of a Application object.
    ///
    /// # Type parameters
    ///
    /// - `S`: A type that implements the `State` trait. e.g. Your initial
    ///        game logic.
    ///
    /// # Lifetimes
    ///
    /// - `a`: The lifetime of the `State` objects.
    /// - `b`: This lifetime is inherited from `specs` and `shred`, it is
    ///        the minimum lifetime of the systems used by `Application`
    ///
    /// # Errors
    ///
    /// Application will return an error if the internal threadpool fails
    /// to initialize correctly because of systems resource limitations
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::transform::{Child, LocalTransform, TransformSystem};
    ///
    /// struct NullState;
    ///
    /// impl State for NullState {}
    ///
    /// fn main() {
    ///     // initialize the builder, the `ApplicationBuilder` object
    ///     // follows the use pattern of most builder objects found
    ///     // in the rust ecosystem. Each function modifies the object
    ///     // returning a new object that with the modified configuration.
    ///     let mut game = Application::build(NullState)
    ///         .expect("Failed to initialize")
    ///
    ///     // components can be registered at this stage
    ///         .register::<Child>()
    ///         .register::<LocalTransform>()
    ///
    ///     // systems can be added before the game is run
    ///         .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
    ///
    ///     // lastly we can build the Application object
    ///         .build()
    ///         .expect("Failed to create Application");
    ///
    ///     // the game instance can now be run, this exits only when the game is done
    ///     game.run();
    /// }
    /// ~~~

    pub fn new(initial_state: T) -> Result<Self> {
        use num_cpus;
        use rayon::Configuration;

        let num_cores = num_cpus::get();
        let cfg = Configuration::new().num_threads(num_cores);
        let pool = ThreadPool::new(cfg).map(|p| Arc::new(p)).map_err(|_| {
            Error::Application
        })?;
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

    /// Registers a component into the entity-component-system. This method
    /// takes no options other than the component type which is defined
    /// using a 'turbofish'. See the example for what this looks like.
    ///
    /// You must register a component type before it can be used. If
    /// code accesses a component that has not previously been registered
    /// it will `panic`.
    ///
    /// # Type Parameters
    ///
    /// - `C`: The Component type that you are registering. This must
    ///        implement the `Component` trait to be registered.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::{Component, HashMapStorage};
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// // define your custom type for the ECS
    /// struct Velocity([f32; 3]);
    ///
    /// // the compiler must be told how to store every component, `Velocity`
    /// // in this case. This is done via The `amethyst::ecs::Component` trait.
    /// impl Component for Velocity {
    ///     // To do this the `Component` trait has an associated type
    ///     // which is used to associate the type back to the container type.
    ///     // There are a few common containers, VecStorage and HashMapStorage
    ///     // are the most common used.
    ///     //
    ///     // See the documentation on the specs::Storage trait for more information.
    ///     // https://docs.rs/specs/0.9.5/specs/struct.Storage.html
    ///     type Storage = HashMapStorage<Velocity>;
    /// }
    ///
    /// fn main() {
    ///     // After creating a builder, we can add any number of components
    ///     // using the register method.
    ///     Application::build(NullState)
    ///         .expect("Failed to initialize")
    ///         .register::<Velocity>();
    ///
    /// }
    /// ~~~
    ///
    pub fn register<C>(mut self) -> Self
    where
        C: Component,
    {
        self.world.register::<C>();
        self
    }

    /// Adds the supplied ECS resource which can be accessed from game systems.
    ///
    /// Resources are common data that is shared with one or more game system.
    ///
    /// If a resource is added with the identical type as an existing resource,
    /// the new resource will replace the old one and the old resource will
    /// be dropped.
    ///
    /// # Parameters
    /// `resource`: The initialized resource you wish to register
    ///
    /// # Type Parameters
    ///
    /// `R`: `resource` must implement the `Resource` trait. This trait will
    ///      be automatically implemented if `Any` + `Send` + `Sync` traits
    ///      exist for type `R`.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// // your resource can be anything that can be safely stored in a `Arc`
    /// // in this example, it is a vector of scores with a user name
    /// struct HighScores(Vec<Score>);
    ///
    /// struct Score {
    ///     score: u32,
    ///     user: String
    /// }
    ///
    /// fn main() {
    ///     let score_board = HighScores(Vec::new());
    ///     Application::build(NullState)
    ///         .expect("Failed to initialize")
    ///         .add_resource(score_board);
    /// }
    /// ~~~
    pub fn add_resource<R>(mut self, resource: R) -> Self
    where
        R: Resource,
    {
        self.world.add_resource(resource);
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
    where
        for<'c> S: System<'c> + Send + 'a + 'b,
    {
        self.disp_builder = self.disp_builder.add(sys, name, dep);
        self
    }

    /// Adds a given thread-local system `sys` to the Application.
    ///
    /// All thread-local systems are executed sequentially after all
    /// non-thread-local systems.
    pub fn with_thread_local<S>(mut self, sys: S) -> Self
    where
        for<'c> S: System<'c> + 'a + 'b,
    {
        self.disp_builder = self.disp_builder.add_thread_local(sys);
        self
    }

    /// Automatically registers components, adds resources and the rendering system.
    pub fn with_renderer(
        mut self,
        pipe: PipelineBuilder,
        config: Option<DisplayConfig>,
    ) -> Result<Self> {
        use ecs::SystemExt;
        use ecs::rendering::RenderSystem;
        let render_sys = RenderSystem::build((&self.events, pipe, config), &mut self.world)?;
        self = self.with_thread_local(render_sys);

        Ok(
            self.register_mesh_asset()
                .register_texture_asset()
                .register_material_not_yet_asset(),
        )


    }

    /// Add asset loader to resources
    pub fn add_store<I, S>(self, name: I, store: S) -> Self
    where
        I: Into<String>,
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
    where
        A: Component + Asset + Clone + Send + Sync + 'static,
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

        #[cfg(feature = "profiler")] register_thread_with_profiler("Main".into());
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
        use ecs::rendering::{MeshComponent, MeshContext, Factory};
        self.register_asset::<MeshComponent, _>(|world| {
            let factory = world.read_resource::<Factory>();
            MeshContext::new((&*factory).clone())
        })
    }

    /// Register new context within the loader
    fn register_material_not_yet_asset(mut self) -> Self {
        use assets::AssetFuture;
        use ecs::rendering::MaterialComponent;
        use specs::common::Merge;

        self.world.register::<MaterialComponent>();
        self.world.register::<AssetFuture<MaterialComponent>>();
        self = self.with(Merge::<AssetFuture<MaterialComponent>>::new(), "", &[]);
        self
    }

    /// Register new context within the loader
    fn register_texture_asset(self) -> Self {
        use ecs::rendering::{TextureComponent, TextureContext, Factory};
        self.register_asset::<TextureComponent, _>(|world| {
            let factory = world.read_resource::<Factory>();
            TextureContext::new((&*factory).clone())
        })
    }
}
