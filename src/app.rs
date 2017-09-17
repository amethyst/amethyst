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

/// An Application is the root object of the game engine. It binds the OS
/// event loop, state machines, timers and other core components in a central place.
///
/// Since Application functions as the root of the game, Amethyst does not need
/// to use any global variables. Within this object is everything that your
/// game needs to run.
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
    /// impl State for NullState {}
    /// 
    /// let mut game = Application::new(NullState).expect("Failed to initialize");
    /// game.run();
    /// ~~~
    pub fn new<S>(initial_state: S) -> Result<Application<'a, 'b>>
    where
        S: State + 'a
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

/// `ApplicationBuilder` is an interface that allows for creation of an [`Application`](struct.Application.html)
/// using a custom set of configuration. This is the normal way an [`Application`](struct.Application.html)
/// object is created.
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
    /// impl State for NullState {}
    /// 
    /// // initialize the builder, the `ApplicationBuilder` object
    /// // follows the use pattern of most builder objects found
    /// // in the rust ecosystem. Each function modifies the object
    /// // returning a new object with the modified configuration.
    /// let mut game = Application::build(NullState)
    ///     .expect("Failed to initialize")
    /// 
    /// // components can be registered at this stage
    ///     .register::<Child>()
    ///     .register::<LocalTransform>()
    /// 
    /// // systems can be added before the game is run
    ///     .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
    /// 
    /// // lastly we can build the Application object
    ///     .build()
    ///     .expect("Failed to create Application");
    /// 
    /// // the game instance can now be run, this exits only when the game is done
    /// game.run();
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
    /// // After creating a builder, we can add any number of components
    /// // using the register method.
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     .register::<Velocity>();
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
    /// - `resource`: The initialized resource you wish to register
    ///
    /// # Type Parameters
    ///
    /// - `R`: `resource` must implement the `Resource` trait. This trait will
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
    /// let score_board = HighScores(Vec::new());
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     .with_resource(score_board);
    ///
    /// ~~~
    pub fn with_resource<R>(mut self, resource: R) -> Self
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
    /// `with_barrier()`. Thread-local systems are not affected by barriers;
    /// they're always executed at the end.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::System;
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, (): Self::SystemData) {}
    /// }
    ///
    /// // Three systems are added in this example. The "tabby cat" & "tom cat"
    /// // systems will both run in parallel. Only after both cat systems have
    /// // run is the "doggo" system permitted to run them.
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     .with(NopSystem, "tabby cat", &[])
    ///     .with(NopSystem, "tom cat", &[])
    ///     .with_barrier()
    ///     .with(NopSystem, "doggo", &[]);
    /// ~~~
    pub fn with_barrier(mut self) -> Self {
        self.disp_builder = self.disp_builder.add_barrier();
        self
    }

    /// Adds a given system to the game loop.
    ///
    /// __Note:__ all dependencies must be added before you add the system.
    ///
    /// # Parameters
    ///
    /// - `system`: The system that is to be added to the game loop.
    /// - `name`: A unique string to identify the system by. This is used for 
    ///         dependency tracking. This name may be empty `""` string in which
    ///         case it cannot be referenced as a dependency.
    /// - `dependencies`: A list of named system that _must_ have completed running
    ///                 before this system is permitted to run.
    ///                 This may be an empty list if there is no dependencies.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `System` trait.
    ///
    /// # Panics
    /// 
    /// If two system are added that share an identical name, this function will panic.
    /// Empty names are permitted, and this function will not panic if more then two are added.
    ///
    /// If a dependency is referenced (by name), but has not previously been added this
    /// function will panic.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::System;
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, _: Self::SystemData) {}
    /// }
    ///
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     // This will add the "foo" system to the game loop, in this case
    ///     // the "foo" system will not depend on any systems.
    ///     .with(NopSystem, "foo", &[])
    ///     // The "bar" system will only run after the "foo" system has completed
    ///     .with(NopSystem, "bar", &["foo"])
    ///     // It is legal to register a system with an empty name
    ///     .with(NopSystem, "", &[]);
    /// ~~~
    pub fn with<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Self
    where
        for<'c> S: System<'c> + Send + 'a + 'b,
    {
        self.disp_builder = self.disp_builder.add(system, name, dependencies);
        self
    }

    /// Add a given thread-local system to the game loop.
    ///
    /// A thread-local system is one that _must_ run on the main thread of the
    /// game. A thread-local system would be necessary typically to work
    /// around vendor APIs that have thread dependent designs; an example
    /// being OpenGL which uses a thread-local state machine to function.
    ///
    /// All thread-local systems are executed sequentially after all
    /// non-thread-local systems.
    ///
    /// # Parameters
    ///
    /// - `system`: The system that is to be added to the game loop.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A type that implements the `System` trait.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::ecs::System;
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// struct NopSystem;
    /// impl<'a> System<'a> for NopSystem {
    ///     type SystemData = ();
    ///     fn run(&mut self, _: Self::SystemData) {}
    /// }
    /// 
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     // the Nop system is registered here
    ///     .with_thread_local(NopSystem);
    /// ~~~
    pub fn with_thread_local<S>(mut self, system: S) -> Self
    where
        for<'c> S: System<'c> + 'a + 'b,
    {
        self.disp_builder = self.disp_builder.add_thread_local(system);
        self
    }

    /// Automatically registers the rendering system and all required components
    /// and resources that the rendering system requires.
    ///
    /// # Parameters
    /// - `pipeline`: A pipeline builder describing the render pipeline.
    /// - `config`: An optional display configuration. If there is no supplied
    ///             display configuration the configuration will use the default
    ///             configuration created by `amethyst::renderer::Config::default()`
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it, this is
    /// wrapped in a `Result`.
    ///
    /// # Errors
    ///
    /// This method initializes the renderer, which uses a number of system APIs
    /// that we cannot guarantee will function correctly on every system.
    ///
    /// Sources of possible error (none exhaustive):
    ///
    /// - Failure to create a window
    /// - Failure to initialize the lower level graphics context (OpenGL, DX)
    /// - Failure to initialize the thread pool
    /// - Failure to compile shaders for the target platform
    /// - linking error between shader variables and framework
    ///
    /// # Examples
    ///
    /// See the `renderable` example supplied in the examples folder for a fully
    /// working example that illustrates how to configure the renderer.
    /// The example below is a brief example showing how and where to get the
    /// types needed to initialize the renderer.
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::renderer;
    /// use amethyst::renderer::prelude::*;
    /// 
    /// struct NullState;
    /// impl State for NullState {}
    /// 
    /// // Create a new display_config, we can tweak the configuration
    /// // here, or load it from a file.
    /// let mut display_config = renderer::Config::default();
    /// display_config.title = "An Amazing Amethyst Game!".into();
    /// display_config.dimensions = Some((640, 480));
    ///
    /// // A pipeline needs to be built to initialize the renderer
    /// let pipeline_builder = Pipeline::build().with_stage(
    ///     Stage::with_backbuffer()
    ///         .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
    ///         .with_model_pass(pass::DrawShaded::<PosNormTex>::new()),
    /// );
    ///     
    /// // finally we are able to add the renderer to our application.
    /// Application::build(NullState)
    ///     .expect("Failed to initialize")
    ///     .with_renderer(pipeline_builder, Some(display_config))
    ///     .expect("Failed to create render system for application");
    /// ~~~
    pub fn with_renderer(
        mut self,
        pipeline: PipelineBuilder,
        config: Option<DisplayConfig>,
    ) -> Result<Self> {
        use ecs::SystemExt;
        use ecs::rendering::RenderSystem;
        let render_sys = RenderSystem::build((&self.events, pipeline, config), &mut self.world)?;
        Ok(
            self.with_thread_local(render_sys)
                .register_mesh_asset()
                .register_texture_asset()
                .register_material_not_yet_asset(),
        )
    }

    /// Register an asset store with the loader logic of the Application.
    ///
    /// If the asset store exists, that shares a name with the new store the net
    /// effect will be a replacement of the older store with the new one.
    /// No warning or panic will result from this action.
    ///
    /// # Parameters
    ///
    /// - `name`: A unique name or key to identify the asset storage location. `name`
    ///           is used later to specify where the asset should be loaded from. 
    /// - `store`: The asset store being registered.
    ///
    /// # Type Parameters
    ///
    /// - `I`: A `String`, or a type that can be converted into a`String`.
    /// - `S`: A `Store` asset loader. Typically this is a [`Directory`](../amethyst_assets/struct.Directory.html).
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Examples
    ///
    /// ~~~no_run
    /// use amethyst::prelude::*;
    /// use amethyst::assets::{Directory, Loader};
    /// use amethyst::assets::formats::meshes::ObjFormat;
    /// use amethyst::ecs::rendering::MeshComponent;
    ///
    /// let mut game = Application::build(LoadingState)
    ///     .expect("Failed to initialize")
    ///     // Register the directory "Game Assets" under the name "resources".
    ///     .with_store("resources", Directory::new("Game Assets"))
    ///     .build()
    ///     .expect("Failed to build game")
    ///     .run();
    ///
    /// struct LoadingState;
    /// impl State for LoadingState {
    ///     fn on_start(&mut self, engine: &mut Engine) {
    ///         let loader = engine.world.read_resource::<Loader>();
    ///         // With the `resources`, load a teapot mesh with the format MeshComponent
    ///         // from the directory that registered above.
    ///         let future = loader.load_from::<MeshComponent, _, _, _>("teapot", ObjFormat, "resources");
    ///     }
    /// }
    /// ~~~
    pub fn with_store<I, S>(self, name: I, store: S) -> Self
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

    /// Register a new asset type with the Application. All required components
    /// related to the storage of this asset type will be registered. Since
    /// Amethyst uses AssetFutures to allow for async content loading, Amethyst
    /// needs to have a system that translates AssetFutures into Components as
    /// they resolve. Amethyst registers a system to accomplish this.
    ///
    /// # Parameters
    ///
    /// `make_context`: A closure that returns an initialized `Asset::Context`
    ///                 object. This is given the a reference to the world object
    ///                 to allow it to find any resources previously registered.
    ///
    /// # Type Parameters
    ///
    /// - `A`: The asset type, an `Asset` in reference to Amethyst is a component
    ///        that implements the [`Asset`](../amethyst_assets/trait.Asset.html) trait.
    /// - `F`: A function that returns the `Asset::Context` context object.
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    ///
    // TODO: Create example of this function. It might be easier to build a large
    //       example of a custom type in the `Asset` trait docs
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

    /// Build an `Application` object using the `ApplicationBuilder` as configured.
    ///
    /// # Returns
    ///
    /// This function returns an Application object wrapped in the Result type.
    ///
    /// # Errors
    /// 
    /// This function currently will not produce an error, returning a result
    /// type was strictly for future possibilities.
    ///
    /// # Notes
    ///
    /// If the "profiler" feature is used, this function will register the thread
    /// that executed this function as the "Main" thread.
    ///
    /// # Examples
    ///
    /// See the [example show for `ApplicationBuilder::new()`](struct.ApplicationBuilder.html#examples)
    /// for an example on how this method is used.
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

    // #########################################################################
    //
    // Internal functions
    //
    // #########################################################################

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
