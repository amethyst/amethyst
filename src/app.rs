//! The core engine framework.

use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use core::ECSBundle;
use fern;
use log::LevelFilter;
use rayon::ThreadPool;
use shred::{Resource, RunNow};
use shrev::{EventChannel, ReaderId};
#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};
use winit::{Event, WindowEvent};

use assets::{Asset, Loader, Source};
use core::frame_limiter::{FrameLimiter, FrameRateLimitConfig, FrameRateLimitStrategy};
use core::timing::{Stopwatch, Time};
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use ecs::common::Errors;
use error::{Error, Result};
use state::{State, StateMachine};
use vergen;

/// An Application is the root object of the game engine. It binds the OS
/// event loop, state machines, timers and other core components in a central place.
///
/// Since Application functions as the root of the game, Amethyst does not need
/// to use any global variables. Within this object is everything that your
/// game needs to run.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Application<'a, 'b> {
    /// The world
    #[derivative(Debug = "ignore")]
    pub world: World,

    #[derivative(Debug = "ignore")]
    dispatcher: Dispatcher<'a, 'b>,
    events_reader_id: ReaderId<Event>,
    states: StateMachine<'a>,
    #[derivative(Debug = "ignore")]
    locals: Vec<Box<for<'c> RunNow<'c> + 'b>>,
    ignore_window_close: bool,
}

impl<'a, 'b> Application<'a, 'b> {
    /// Creates a new Application with the given initial game state.
    /// This will create and allocate all the needed resources for
    /// the event loop of the game engine. It is a shortcut for convenience
    /// if you need more control over how the engine is configured you should
    /// be using [build](struct.Application.html#method.build) instead.
    ///
    /// # Parameters
    ///
    /// - `path`: The default path for asset loading.
    ///
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
    /// - `P`: The path type for your standard asset path.
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
    /// Application will return an error if the internal thread pool fails
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
    /// let mut game = Application::new("assets/", NullState).expect("Failed to initialize");
    /// game.run();
    /// ~~~
    pub fn new<P, S>(path: P, initial_state: S) -> Result<Application<'a, 'b>>
    where
        P: AsRef<Path>,
        S: State + 'a,
    {
        ApplicationBuilder::new(path, initial_state)?.build()
    }

    /// Creates a new ApplicationBuilder with the given initial game state.
    ///
    /// This is identical in function to
    /// [ApplicationBuilder::new](struct.ApplicationBuilder.html#method.new).
    pub fn build<P, S>(path: P, initial_state: S) -> Result<ApplicationBuilder<'a, 'b, S>>
    where
        P: AsRef<Path>,
        S: State + 'a,
    {
        ApplicationBuilder::new(path, initial_state)
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
        self.world.write_resource::<Stopwatch>().start();
        while self.states.is_running() {
            self.advance_frame();

            self.world.write_resource::<FrameLimiter>().wait();
            {
                let elapsed = self.world.read_resource::<Stopwatch>().elapsed();
                let mut time = self.world.write_resource::<Time>();
                time.increment_frame_number();
                time.set_delta_time(elapsed);
            }
            let mut stopwatch = self.world.write_resource::<Stopwatch>();
            stopwatch.stop();
            stopwatch.restart();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("initialize");
        self.states.start(&mut self.world);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        trace!("Advancing frame (`Application::advance_frame`)");

        {
            let world = &mut self.world;
            let states = &mut self.states;
            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            let events = world
                .read_resource::<EventChannel<Event>>()
                .read(&mut self.events_reader_id)
                .cloned()
                .collect::<Vec<_>>();

            for event in events {
                states.handle_event(world, event.clone());
                if !self.ignore_window_close {
                    if let &Event::WindowEvent {
                        event: WindowEvent::Closed,
                        ..
                    } = &event
                    {
                        states.stop(world);
                    }
                }
            }
        }
        {
            let do_fixed = {
                let time = self.world.write_resource::<Time>();
                time.last_fixed_update().elapsed() >= time.fixed_time()
            };
            #[cfg(feature = "profiler")]
            profile_scope!("fixed_update");
            if do_fixed {
                self.states.fixed_update(&mut self.world);
                self.world.write_resource::<Time>().finish_fixed_update();
            }

            #[cfg(feature = "profiler")]
            profile_scope!("update");
            self.states.update(&mut self.world);
        }

        #[cfg(feature = "profiler")]
        profile_scope!("dispatch");
        self.dispatcher.dispatch(&mut self.world.res);

        for local in &mut self.locals {
            local.run_now(&self.world.res);
        }

        #[cfg(feature = "profiler")]
        profile_scope!("maintain");
        self.world.maintain();

        // TODO: replace this with a more customizable method.
        // TODO: effectively, the user should have more control over error handling here
        // TODO: because right now the app will just exit in case of an error.
        self.world.write_resource::<Errors>().print_and_exit();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        info!("Engine is shutting down");

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
pub struct ApplicationBuilder<'a, 'b, T> {
    // config: Config,
    disp_builder: DispatcherBuilder<'a, 'b>,
    initial_state: T,
    /// Used by bundles to access the world directly
    pub world: World,
    locals: Vec<Box<for<'c> RunNow<'c> + 'b>>,
    ignore_window_close: bool,
}

impl<'a, 'b, T> ApplicationBuilder<'a, 'b, T> {
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
    /// use amethyst::core::transform::{Parent, LocalTransform, TransformSystem};
    ///
    /// struct NullState;
    /// impl State for NullState {}
    ///
    /// // initialize the builder, the `ApplicationBuilder` object
    /// // follows the use pattern of most builder objects found
    /// // in the rust ecosystem. Each function modifies the object
    /// // returning a new object with the modified configuration.
    /// let mut game = Application::build("assets/", NullState)
    ///     .expect("Failed to initialize")
    ///
    /// // components can be registered at this stage
    ///     .register::<Parent>()
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

    pub fn new<P: AsRef<Path>>(path: P, initial_state: T) -> Result<Self> {
        use bundle::AppBundle;
        use rustc_version_runtime;

        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(LevelFilter::Debug)
            .chain(io::stdout())
            .apply()
            .unwrap_or_else(|e| warn!("Application tried to override existing logger: {}", e));

        info!("Initializing Amethyst...");
        info!("Version: {}", vergen::semver());
        info!("Platform: {}", vergen::target());
        info!("Amethyst git commit: {}", vergen::sha());
        let rustc_meta = rustc_version_runtime::version_meta();
        info!(
            "Rustc version: {} {:?}",
            rustc_meta.semver, rustc_meta.channel
        );
        if let Some(hash) = rustc_meta.commit_hash {
            info!("Rustc git commit: {}", hash);
        }

        let mut disp_builder = DispatcherBuilder::new();
        let mut world = World::new();
        disp_builder = AppBundle::new(path).build(&mut world, disp_builder)?;

        Ok(ApplicationBuilder {
            disp_builder,
            initial_state,
            world,
            locals: Vec::default(),
            ignore_window_close: false,
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
    /// Application::build("assets/", NullState)
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
    /// Application::build("assets/", NullState)
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
    /// Application::build("assets/", NullState)
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
    /// Application::build("assets/", NullState)
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
        for<'c> S: System<'c> + Send + 'a,
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
    /// Application::build("assets/", NullState)
    ///     .expect("Failed to initialize")
    ///     // the Nop system is registered here
    ///     .with_thread_local(NopSystem);
    /// ~~~
    pub fn with_thread_local<S>(mut self, system: S) -> Self
    where
        for<'c> S: System<'c> + 'b,
    {
        self.disp_builder = self.disp_builder.add_thread_local(system);
        self
    }

    /// Add a local `RunNow` system.
    ///
    /// The added system will be dispatched after all normal
    /// and thread local systems. This is special because it does
    /// accept types implementing only `RunNow`, but not
    /// `System`, which is needed for e.g. the `RenderSystem`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amethyst::core::transform::Transform;
    /// use amethyst::prelude::*;
    /// use amethyst::renderer::*;
    ///
    /// # struct Example;
    /// # impl State for Example {}
    /// #
    /// # fn run() -> Result<(), ::amethyst::Error> {
    /// let pipe = Pipeline::build().with_stage(
    ///     Stage::with_backbuffer()
    ///         .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
    ///         .with_pass(DrawShaded::<PosNormTex>::new()),
    /// );
    ///
    /// let config = DisplayConfig::load("config_path.ron");
    ///
    /// let mut game = Application::build("resources/", Example)?
    /// .with_bundle(RenderBundle::new())?
    /// .with_local(RenderSystem::build(pipe, Some(config))?)
    /// .build()?;
    /// # Ok(())
    /// # }
    /// # fn main() { run().unwrap(); }
    /// ```
    pub fn with_local<S>(mut self, system: S) -> Self
    where
        for<'c> S: RunNow<'c> + 'b,
    {
        self.locals.push(Box::new(system));
        self
    }

    /// Add a given ECS bundle to the game loop.
    ///
    /// A bundle is a container for registering a bunch of ECS systems and their dependent
    /// resources and components.
    ///
    /// # Parameters
    ///
    /// - `bundle`: The bundle to add
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it, this is
    /// wrapped in a `Result`.
    ///
    /// # Errors
    ///
    /// This function creates systems and resources, which use any number of dependent
    /// crates or APIs, which could result in any number of errors.
    /// See each individual bundle for a description of the errors it could produce.
    ///
    pub fn with_bundle<B>(mut self, bundle: B) -> Result<Self>
    where
        B: ECSBundle<'a, 'b>,
    {
        self.disp_builder = bundle
            .build(&mut self.world, self.disp_builder)
            .map_err(|err| Error::Core(err))?;
        Ok(self)
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
    /// use amethyst::renderer::ObjFormat;
    /// use amethyst::ecs::World;
    ///
    /// let mut game = Application::build("assets/", LoadingState)
    ///     .expect("Failed to initialize")
    ///     // Register the directory "custom_directory" under the name "resources".
    ///     .with_source("custom_store", Directory::new("custom_directory"))
    ///     .build()
    ///     .expect("Failed to build game")
    ///     .run();
    ///
    /// struct LoadingState;
    /// impl State for LoadingState {
    ///     fn on_start(&mut self, world: &mut World) {
    ///         let storage = world.read_resource();
    ///
    ///         let loader = world.read_resource::<Loader>();
    ///         // Load a teapot mesh from the directory that registered above.
    ///         let mesh = loader.load_from("teapot", ObjFormat, (), "custom_directory",
    ///                                     (), &storage);
    ///     }
    /// }
    /// ~~~
    pub fn with_source<I, S>(self, name: I, store: S) -> Self
    where
        I: Into<String>,
        S: Source,
    {
        {
            let mut loader = self.world.write_resource::<Loader>();
            loader.add_source(name, store);
        }
        self
    }

    /// Sets the maximum frames per second of this game.
    ///
    /// # Parameters
    ///
    /// `strategy`: the frame limit strategy to use
    /// `max_fps`: the maximum frames per second this game will run at.
    ///
    /// # Returns
    ///
    /// This function returns the ApplicationBuilder after modifying it.
    pub fn with_frame_limit(mut self, strategy: FrameRateLimitStrategy, max_fps: u32) -> Self {
        self.world
            .add_resource(FrameLimiter::new(strategy, max_fps));
        self
    }

    /// Sets the maximum frames per second of this game, based on the given config.
    ///
    /// # Parameters
    ///
    /// `config`: the frame limiter config
    ///
    /// # Returns
    ///
    /// This function returns the ApplicationBuilder after modifying it.
    pub fn with_frame_limit_config(mut self, config: FrameRateLimitConfig) -> Self {
        self.world.add_resource(FrameLimiter::from_config(config));
        self
    }

    /// Sets the duration between fixed updates, defaults to one sixtieth of a second.
    ///
    /// # Parameters
    ///
    /// `duration`: The duration between fixed updates.
    ///
    /// # Returns
    ///
    /// This function returns the ApplicationBuilder after modifying it.
    pub fn with_fixed_step_length(self, duration: Duration) -> Self {
        self.world.write_resource::<Time>().set_fixed_time(duration);
        self
    }

    /// Tells the resulting application window to ignore close events if ignore is true.
    /// This will make your game window unresponsive to operating system close commands.
    /// Use with caution.
    ///
    /// # Parameters
    ///
    /// `ignore`: Whether or not the window should ignore these events.  False by default.
    ///
    /// # Returns
    ///
    /// This function returns the ApplicationBuilder after modifying it.
    pub fn ignore_window_close(mut self, ignore: bool) -> Self {
        self.ignore_window_close = ignore;
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
    pub fn register_asset<A>(mut self) -> Self
    where
        A: Asset,
    {
        use assets::{AssetStorage, Handle};

        self.world.add_resource(AssetStorage::<A>::new());
        self.world.register::<Handle<A>>();

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
    pub fn build(self) -> Result<Application<'a, 'b>>
    where
        T: State + 'a,
    {
        trace!("Entering `ApplicationBuilder::build`");

        #[cfg(feature = "profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        let pool = self.world.read_resource::<Arc<ThreadPool>>().clone();
        let reader_id = self.world
            .write_resource::<EventChannel<Event>>()
            .register_reader();

        Ok(Application {
            world: self.world,
            // config: self.config,
            states: StateMachine::new(self.initial_state),
            events_reader_id: reader_id,
            dispatcher: self.disp_builder.with_pool(pool).build(),
            locals: self.locals,
            ignore_window_close: self.ignore_window_close,
        })
    }
}
