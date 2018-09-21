//! The core engine framework.

use amethyst_ui::UiEvent;
use assets::{Loader, Source};
use core::frame_limiter::{FrameLimiter, FrameRateLimitConfig, FrameRateLimitStrategy};
use core::shrev::{EventChannel, ReaderId};
use core::timing::{Stopwatch, Time};
use core::Named;
use ecs::common::Errors;
use ecs::prelude::{Component, World};
use error::{Error, Result};
use game_data::DataInit;
use log::Level;
use rayon::ThreadPoolBuilder;
use shred::Resource;
use state::{State, StateData, StateMachine};
use state_event::StateEvent;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};
use winit::{Event, WindowEvent};

/// An Application is the root object of the game engine. It binds the OS
/// event loop, state machines, timers and other core components in a central place.
///
/// Since Application functions as the root of the game, Amethyst does not need
/// to use any global variables. Within this object is everything that your
/// game needs to run.
///
/// # Logging
///
/// Amethyst performs logging internally using the [log] crate. By default, `Application` will
/// initialize a global logger that simply sends logs to the console. You can take advantage of
/// this and use the logging macros in `log` once you've created your `Application` instance:
///
/// ```
/// extern crate amethyst;
/// #[macro_use]
/// extern crate log;
///
/// use amethyst::prelude::*;
/// use amethyst::core::transform::{Parent, Transform};
/// use amethyst::ecs::prelude::System;
///
/// struct NullState;
/// impl EmptyState for NullState {}
///
/// fn main() -> amethyst::Result<()> {
///     // Build the application instance to initialize the default logger.
///     let mut game = Application::build("assets/", NullState)?
///         .build(())?;
///
///     // Now logging can be performed as normal.
///     info!("Using the default logger provided by amethyst");
///     warn!("Uh-oh, something went wrong!");
///
///     Ok(())
/// }
/// ```
///
/// You can also setup your own logging system. Simply intialize any global logger that supports
/// [log], and it will be used instead of the default logger:
///
/// ```
/// extern crate amethyst;
/// #[macro_use]
/// extern crate log;
/// extern crate env_logger;
///
/// use amethyst::prelude::*;
/// use amethyst::core::transform::{Parent, Transform};
/// use amethyst::ecs::prelude::System;
///
/// struct NullState;
/// impl EmptyState for NullState {}
///
/// fn main() -> amethyst::Result<()> {
///     // Initialize your custom logger (using env_logger in this case) before creating the
///     // `Application` instance.
///     env_logger::init();
///
///     // The default logger will be automatically disabled and any logging amethyst does
///     // will go through your custom logger.
///     let mut game = Application::build("assets/", NullState)?
///         .build(())?;
///
///     Ok(())
/// }
/// ```
///
/// [log]: https://crates.io/crates/log
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Application<'a, T, E: Send + Sync + 'static> {
    /// The world
    #[derivative(Debug = "ignore")]
    world: World,
    window_reader_id: ReaderId<Event>,
    ui_reader_id: ReaderId<UiEvent>,
    custom_reader_id: ReaderId<E>,
    states: StateMachine<'a, T, E>,
    ignore_window_close: bool,
    data: T,
}

impl<'a, T, E: Send + Sync + Clone + 'static> Application<'a, T, E> {
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
    /// impl EmptyState for NullState {}
    ///
    /// let mut game = Application::new("assets/", NullState, ()).expect("Failed to initialize");
    /// game.run();
    /// ~~~
    pub fn new<P, S, I>(path: P, initial_state: S, init: I) -> Result<Self>
    where
        P: AsRef<Path>,
        S: State<T, E> + 'a,
        I: DataInit<T>,
    {
        ApplicationBuilder::new(path, initial_state)?.build(init)
    }

    /// Creates a new ApplicationBuilder with the given initial game state.
    ///
    /// This is identical in function to
    /// [ApplicationBuilder::new](struct.ApplicationBuilder.html#method.new).
    pub fn build<P, S>(path: P, initial_state: S) -> Result<ApplicationBuilder<S, E>>
    where
        P: AsRef<Path>,
        S: State<T, E>,
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
        self.states
            .start(StateData::new(&mut self.world, &mut self.data))
            .expect("Tried to start state machine without any states present");
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        trace!("Advancing frame (`Application::advance_frame`)");

        {
            let world = &mut self.world;
            let states = &mut self.states;
            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            let window_events = world
                .read_resource::<EventChannel<Event>>()
                .read(&mut self.window_reader_id)
                .cloned()
                .map(|e| StateEvent::Window(e))
                .collect::<Vec<_>>();

            for event in window_events {
                if !self.ignore_window_close {
                    if cfg!(target_os = "ios") {
                        if let &StateEvent::Window(Event::WindowEvent {
                            event: WindowEvent::Destroyed,
                            ..
                        }) = &event
                        {
                            states.stop(StateData::new(world, &mut self.data));
                        }
                    } else {
                        if let &StateEvent::Window(Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            ..
                        }) = &event
                        {
                            states.stop(StateData::new(world, &mut self.data));
                        }
                    }
                }
                states.handle_event(StateData::new(world, &mut self.data), event);
            }

            let ui_events = world
                .read_resource::<EventChannel<UiEvent>>()
                .read(&mut self.ui_reader_id)
                .cloned()
                .map(|e| StateEvent::Ui(e))
                .collect::<Vec<_>>();
            for event in ui_events {
                states.handle_event(StateData::new(world, &mut self.data), event);
            }

            let custom_events = world
                .read_resource::<EventChannel<E>>()
                .read(&mut self.custom_reader_id)
                .cloned()
                .map(|e| StateEvent::Custom(e))
                .collect::<Vec<_>>();
            for event in custom_events {
                states.handle_event(StateData::new(world, &mut self.data), event);
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
                self.states
                    .fixed_update(StateData::new(&mut self.world, &mut self.data));
                self.world.write_resource::<Time>().finish_fixed_update();
            }

            #[cfg(feature = "profiler")]
            profile_scope!("update");
            self.states
                .update(StateData::new(&mut self.world, &mut self.data));
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
impl<'a, T, E: Send + Sync + 'static> Drop for Application<'a, T, E> {
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        use utils::application_root_dir;
        let path = format!("{}/thread_profile.json", application_root_dir());
        write_profile(path.as_str());
    }
}

/// `ApplicationBuilder` is an interface that allows for creation of an
/// [`Application`](struct.Application.html)
/// using a custom set of configuration. This is the normal way an
/// [`Application`](struct.Application.html)
/// object is created.
pub struct ApplicationBuilder<S, E> {
    // config: Config,
    initial_state: S,
    /// Used by bundles to access the world directly
    pub world: World,
    ignore_window_close: bool,
    phantom: PhantomData<E>,
}

impl<S, E: Send + Sync + 'static> ApplicationBuilder<S, E> {
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
    /// use amethyst::core::transform::{Parent, Transform};
    /// use amethyst::ecs::prelude::System;
    ///
    /// struct NullState;
    /// impl EmptyState for NullState {}
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
    ///     .register::<Transform>()
    ///
    /// // lastly we can build the Application object
    /// // the `build` function takes the user defined game data initializer as input
    ///     .build(())
    ///     .expect("Failed to create Application");
    ///
    /// // the game instance can now be run, this exits only when the game is done
    /// game.run();
    /// ~~~

    pub fn new<P: AsRef<Path>>(path: P, initial_state: S) -> Result<Self> {
        use rustc_version_runtime;

        if !log_enabled!(Level::Error) {
            eprintln!(
                "WARNING: No logger detected! Did you forget to call `amethyst::start_logger()`?"
            );
        }

        info!("Initializing Amethyst...");
        info!("Version: {}", env!("CARGO_PKG_VERSION"));
        info!("Platform: {}", env!("VERGEN_TARGET_TRIPLE"));
        info!("Amethyst git commit: {}", env!("VERGEN_SHA"));
        let rustc_meta = rustc_version_runtime::version_meta();
        info!(
            "Rustc version: {} {:?}",
            rustc_meta.semver, rustc_meta.channel
        );
        if let Some(hash) = rustc_meta.commit_hash {
            info!("Rustc git commit: {}", hash);
        }

        let mut world = World::new();

        let thread_pool_builder = ThreadPoolBuilder::new();
        #[cfg(feature = "profiler")]
        let thread_pool_builder = thread_pool_builder.start_handler(|index| {
            register_thread_with_profiler();
        });
        let pool = thread_pool_builder
            .build()
            .map(|p| Arc::new(p))
            .map_err(|err| Error::Core(err.description().to_string().into()))?;
        world.add_resource(Loader::new(path.as_ref().to_owned(), pool.clone()));
        world.add_resource(pool);
        world.add_resource(EventChannel::<Event>::with_capacity(2000));
        world.add_resource(EventChannel::<UiEvent>::with_capacity(40));
        world.add_resource(EventChannel::<E>::with_capacity(2));
        world.add_resource(Errors::default());
        world.add_resource(FrameLimiter::default());
        world.add_resource(Stopwatch::default());
        world.add_resource(Time::default());

        world.register::<Named>();

        Ok(ApplicationBuilder {
            initial_state,
            world,
            ignore_window_close: false,
            phantom: PhantomData,
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
    /// use amethyst::ecs::prelude::Component;
    /// use amethyst::ecs::storage::HashMapStorage;
    ///
    /// struct NullState;
    /// impl EmptyState for NullState {}
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
        C::Storage: Default,
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
    /// impl EmptyState for NullState {}
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
    /// use amethyst::ecs::prelude::World;
    ///
    /// let mut game = Application::build("assets/", LoadingState)
    ///     .expect("Failed to initialize")
    ///     // Register the directory "custom_directory" under the name "resources".
    ///     .with_source("custom_store", Directory::new("custom_directory"))
    ///     .build(GameDataBuilder::default())
    ///     .expect("Failed to build game")
    ///     .run();
    ///
    /// struct LoadingState;
    /// impl<'a, 'b> SimpleState<'a, 'b> for LoadingState {
    ///     fn on_start(&mut self, data: StateData<GameData>) {
    ///         let storage = data.world.read_resource();
    ///
    ///         let loader = data.world.read_resource::<Loader>();
    ///         // Load a teapot mesh from the directory that registered above.
    ///         let mesh = loader.load_from("teapot", ObjFormat, (), "custom_directory",
    ///                                     (), &storage);
    ///     }
    /// }
    /// ~~~
    pub fn with_source<I, O>(self, name: I, store: O) -> Self
    where
        I: Into<String>,
        O: Source,
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
    pub fn build<'a, T, I>(mut self, init: I) -> Result<Application<'a, T, E>>
    where
        S: State<T, E> + 'a,
        I: DataInit<T>,
    {
        trace!("Entering `ApplicationBuilder::build`");

        #[cfg(feature = "profiler")]
        register_thread_with_profiler();
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        let window_reader_id = self
            .world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        let ui_reader_id = self
            .world
            .write_resource::<EventChannel<UiEvent>>()
            .register_reader();
        let custom_reader_id = self
            .world
            .write_resource::<EventChannel<E>>()
            .register_reader();

        let data = init.build(&mut self.world);

        Ok(Application {
            world: self.world,
            states: StateMachine::new(self.initial_state),
            window_reader_id,
            ui_reader_id,
            custom_reader_id,
            ignore_window_close: self.ignore_window_close,
            data,
        })
    }
}
