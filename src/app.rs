//! The core engine framework.

use std::{env, marker::PhantomData, path::Path, sync::Arc, time::Duration};

use derivative::Derivative;
use log::{debug, info, log_enabled, trace, Level};
use rayon::ThreadPoolBuilder;
#[cfg(feature = "profiler")]
use thread_profiler::{profile_scope, register_thread_with_profiler, write_profile};
use winit::event::{Event, WindowEvent};

use crate::{
    assets::{start_asset_daemon, DefaultLoader, Source},
    core::{
        frame_limiter::{FrameLimiter, FrameRateLimitConfig, FrameRateLimitStrategy},
        shrev::{EventChannel, ReaderId},
        ArcThreadPool, EventReader, Stopwatch, Time,
    },
    ecs::*,
    error::Error,
    game_data::{DataDispose, DataInit},
    state::{State, StateData, StateMachine, TransEvent},
    state_event::{StateEvent, StateEventReader},
};

/// `CoreApplication` is the application implementation for the game engine. This is fully generic
/// over the state type and event type.
///
/// When starting out with Amethyst, use the type alias `Application`, which have sensible defaults
/// for the `Event` and `EventReader` generic types.
///
/// ### Type parameters:
///
/// - `T`: `State`
/// - `E`: `Event` type that should be sent to the states
/// - `R`: `EventReader` implementation for the given event type `E`
#[derive(Derivative)]
#[derivative(Debug)]
pub struct CoreApplication<'a, T, E = StateEvent, R = StateEventReader>
where
    T: DataDispose + 'static,
    E: 'static,
{
    /// The world
    #[derivative(Debug = "ignore")]
    world: World,
    #[derivative(Debug = "ignore")]
    resources: Resources,
    #[derivative(Debug = "ignore")]
    reader: R,
    #[derivative(Debug = "ignore")]
    events: Vec<E>,
    #[derivative(Debug = "ignore")]
    event_reader_id: ReaderId<Event<'static, ()>>,
    #[derivative(Debug = "ignore")]
    trans_reader_id: ReaderId<TransEvent<T, E>>,
    states: StateMachine<'a, T, E>,
    ignore_window_close: bool,
    data: T,
}

/// An Application is the root object of the game engine. It binds the OS
/// event loop, state machines, timers and other core components in a central place.
///
/// Since Application functions as the root of the game, Amethyst does not need
/// to use any global variables. Within this object is everything that your
/// game needs to run.
///
/// # Logging
///
/// Amethyst performs logging internally using the [log] crate. By default, `CoreApplication`
/// will initialize a global logger that simply sends logs to the console.
/// You can take advantage of this and use the logging macros in `log` once
/// you've created your `CoreApplication` instance:
///
/// ```
/// use amethyst::{
///     core::transform::{Parent, Transform},
///     prelude::*,
/// };
/// use log::{info, warn};
///
/// struct NullState;
/// impl EmptyState for NullState {}
///
/// fn main() -> amethyst::Result<()> {
///     amethyst::start_logger(Default::default());
///
///     // Build the application instance to initialize the default logger.
///     let assets_dir = "assets/";
///     let game = Application::build(assets_dir, NullState)?.build(())?;
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
/// use amethyst::{
///     core::transform::{Parent, Transform},
///     prelude::*,
/// };
/// use env_logger;
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
///     let assets_dir = "assets/";
///     let game = Application::build(assets_dir, NullState)?.build(())?;
///
///     Ok(())
/// }
/// ```
///
/// [log]: https://crates.io/crates/log
pub type Application<'a, T> = CoreApplication<'a, T, StateEvent, StateEventReader>;

impl<'a, T, E, R> CoreApplication<'static, T, E, R>
where
    T: DataDispose + 'static,
    E: Clone + Send + Sync + 'static,
    R: EventReader<Event = E> + 'static,
{
    /// Creates a new CoreApplication with the given initial game state.
    /// This will create and allocate all the needed resources for
    /// the event loop of the game engine. It is a shortcut for convenience
    /// if you need more control over how the engine is configured you should
    /// be using [build](struct.CoreApplication.html#method.build) instead.
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
    /// Returns a `Result` type wrapping the `CoreApplication` type. See
    /// [errors](struct.CoreApplication.html#errors) for a full list of
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
    ///        the minimum lifetime of the systems used by `CoreApplication`
    ///
    /// # Errors
    ///
    /// Application will return an error if the internal thread pool fails
    /// to initialize correctly because of systems resource limitations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amethyst::prelude::*;
    ///
    /// struct NullState;
    /// impl EmptyState for NullState {}
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// #
    /// let assets_dir = "assets/";
    /// let game = Application::build(assets_dir, NullState)?.build(())?;
    /// game.run();
    ///
    /// #  Ok(())
    /// # }
    /// ```
    pub fn new<P, S, I>(path: P, initial_state: S, init: I) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: State<T, E> + 'static,
        I: DataInit<T>,
        R: EventReader<Event = E> + Default,
    {
        ApplicationBuilder::new(path, initial_state)?.build(init)
    }

    /// Creates a new ApplicationBuilder with the given initial game state.
    ///
    /// This is identical in function to
    /// [ApplicationBuilder::new](struct.ApplicationBuilder.html#method.new).
    pub fn build<P, S>(path: P, initial_state: S) -> Result<ApplicationBuilder<S, T, E, R>, Error>
    where
        P: AsRef<Path>,
        S: State<T, E> + 'a,
        R: EventReader<Event = E>,
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
    pub fn run(mut self) {
        #[cfg(feature = "sentry")]
        let _sentry_guard = if let Some(dsn) = option_env!("SENTRY_DSN") {
            let guard = sentry::init(dsn);
            Some(guard)
        } else {
            None
        };

        self.initialize();

        self.resources.get_mut::<Stopwatch>().unwrap().start();

        while self.states.is_running() {
            self.advance_frame();
            {
                #[cfg(feature = "profiler")]
                profile_scope!("frame_limiter wait");
                self.resources.get_mut::<FrameLimiter>().unwrap().wait();
            }
            {
                let mut stopwatch = self.resources.get_mut::<Stopwatch>().unwrap();
                let elapsed = stopwatch.elapsed();
                let mut time = self.resources.get_mut::<Time>().unwrap();
                time.advance_frame(elapsed);
                stopwatch.stop();
                stopwatch.restart();
            }
        }
        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        #[cfg(feature = "profiler")]
        profile_scope!("initialize");
        self.states
            .start(StateData::new(
                &mut self.world,
                &mut self.resources,
                &mut self.data,
            ))
            .expect("Tried to start state machine without any states present");
    }

    // React to window close events
    fn should_close(&mut self) -> bool {
        if self.ignore_window_close {
            false
        } else {
            let reader_id = &mut self.event_reader_id;
            self.resources
                .get_mut::<EventChannel<Event<'_, ()>>>()
                .unwrap()
                .read(reader_id)
                .any(|e| {
                    if cfg!(target_os = "ios") {
                        matches!(
                            e,
                            Event::WindowEvent {
                                event: WindowEvent::Destroyed,
                                ..
                            }
                        )
                    } else {
                        matches!(
                            e,
                            Event::WindowEvent {
                                event: WindowEvent::CloseRequested,
                                ..
                            }
                        )
                    }
                })
        }
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        trace!("Advancing frame (`Application::advance_frame`)");
        if self.should_close() {
            let world = &mut self.world;
            let resources = &mut self.resources;
            let states = &mut self.states;
            states.stop(StateData::new(world, resources, &mut self.data));
        }

        // Read the Trans queue and apply changes.

        let world = &mut self.world;
        let resources = &mut self.resources;
        let states = &mut self.states;
        let reader = &mut self.trans_reader_id;

        let trans = resources
            .get_mut::<EventChannel<TransEvent<T, E>>>()
            .unwrap()
            .read(reader)
            .map(|e| e())
            .collect::<Vec<_>>();
        for tr in trans {
            states.transition(tr, StateData::new(world, resources, &mut self.data));
        }

        {
            #[cfg(feature = "profiler")]
            profile_scope!("handle_event");

            self.reader.read(resources, &mut self.events);

            for e in self.events.drain(..) {
                states.handle_event(StateData::new(world, resources, &mut self.data), e);
            }
        }

        {
            #[cfg(feature = "profiler")]
            profile_scope!("fixed_update");

            while {
                self.resources
                    .get_mut::<Time>()
                    .unwrap()
                    .step_fixed_update()
            } {
                self.states.fixed_update(StateData::new(
                    &mut self.world,
                    &mut self.resources,
                    &mut self.data,
                ));
            }
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("update");
            self.states.update(StateData::new(
                &mut self.world,
                &mut self.resources,
                &mut self.data,
            ));
        }

        #[cfg(feature = "profiler")]
        profile_scope!("maintain");
        // TODO: do defrag here?
        //self.world.maintain();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        info!("Engine is shutting down");
        self.data.dispose(&mut self.world, &mut self.resources);
    }
}

#[cfg(feature = "profiler")]
impl<'a, T, E, R> Drop for CoreApplication<'a, T, E, R>
where
    T: DataDispose,
{
    fn drop(&mut self) {
        // TODO: Specify filename in config.
        use crate::utils::application_root_dir;
        let app_root = application_root_dir().expect("application root dir to exist");
        let path = app_root.join("thread_profile.json");
        write_profile(path.to_str().expect("application root dir to be a string"));
    }
}

/// `ApplicationBuilder` is an interface that allows for creation of an
/// [`CoreApplication`](struct.CoreApplication.html)
/// using a custom set of configuration. This is the normal way an
/// [`CoreApplication`](struct.CoreApplication.html)
/// object is created.
#[allow(missing_debug_implementations)]
pub struct ApplicationBuilder<S, T, E, R> {
    // config: Config,
    initial_state: S,
    /// Used by bundles to initialize any entities in the world
    pub world: World,
    /// Used by bundles to initialize any resources in the world
    pub resources: Resources,
    ignore_window_close: bool,
    phantom: PhantomData<(T, E, R)>,
}

impl<S, T, E, X> ApplicationBuilder<S, T, E, X>
where
    T: DataDispose + 'static,
    E: 'static,
{
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
    /// Returns a `Result` type wrapping the `CoreApplication` type. See
    /// [errors](struct.CoreApplication.html#errors) for a full list of
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
    ///        the minimum lifetime of the systems used by `CoreApplication`
    ///
    /// # Errors
    ///
    /// CoreApplication will return an error if the internal threadpool fails
    /// to initialize correctly because of systems resource limitations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amethyst::{
    ///     core::transform::{Parent, Transform},
    ///     prelude::*,
    /// };
    ///
    /// struct NullState;
    /// impl EmptyState for NullState {}
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// #
    /// // initialize the builder, the `ApplicationBuilder` object
    /// // follows the use pattern of most builder objects found
    /// // in the rust ecosystem. Each function modifies the object
    /// // returning a new object with the modified configuration.
    /// let assets_dir = "assets/";
    /// let game = Application::build(assets_dir, NullState)?
    ///     // lastly we can build the Application object
    ///     // the `build` function takes the user defined game data initializer as input
    ///     .build(())?;
    ///
    /// // the game instance can now be run, this exits only when the game is done
    /// game.run();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P, initial_state: S) -> Result<Self, Error> {
        if !log_enabled!(Level::Error) {
            eprintln!(
                "WARNING: No logger detected! Did you forget to call `amethyst::start_logger()`?"
            );
        }

        info!("Initializing Amethyst...");
        info!("Version: {}", env!("CARGO_PKG_VERSION"));
        info!("Platform: {}", env!("VERGEN_TARGET_TRIPLE"));
        info!("Amethyst git commit: {}", env!("VERGEN_SHA"));
        #[cfg(feature = "sentry")]
        {
            if let Some(sentry) = option_env!("SENTRY_DSN") {
                info!("Sentry DSN: {}", sentry);
            }
        }

        let rustc_meta = rustc_version_runtime::version_meta();
        info!(
            "Rustc version: {} {:?}",
            rustc_meta.semver, rustc_meta.channel
        );
        if let Some(hash) = rustc_meta.commit_hash {
            info!("Rustc git commit: {}", hash);
        }

        let asset_dirs = vec![path.as_ref().to_path_buf()];
        start_asset_daemon(asset_dirs);

        let thread_count: Option<usize> = env::var("AMETHYST_NUM_THREADS")
            .as_ref()
            .map(|s| {
                s.as_str()
                    .parse()
                    .expect("AMETHYST_NUM_THREADS was provided but is not a valid number!")
            })
            .ok();

        let world = World::default();
        let mut resources = Resources::default();

        let thread_pool_builder = ThreadPoolBuilder::new();
        #[cfg(feature = "profiler")]
        let thread_pool_builder = thread_pool_builder.start_handler(|_index| {
            register_thread_with_profiler();
        });
        let pool: ArcThreadPool;
        if let Some(thread_count) = thread_count {
            debug!("Running Amethyst with fixed thread pool: {}", thread_count);
            pool = thread_pool_builder
                .num_threads(thread_count)
                .build()
                .map(Arc::new)?;
        } else {
            pool = thread_pool_builder.build().map(Arc::new)?;
        }
        // FIXME check that the loader is added to the resources
        // resources.insert(Loader::new(path.as_ref().to_owned(), pool.clone()));
        resources.insert(pool);
        resources.insert(EventChannel::<Event<'static, ()>>::with_capacity(2000));
        //resources.insert(EventChannel::<UiEvent>::with_capacity(40));
        resources.insert(FrameLimiter::default());
        resources.insert(Stopwatch::default());
        resources.insert(Time::default());

        Ok(Self {
            initial_state,
            world,
            resources,
            ignore_window_close: false,
            phantom: PhantomData,
        })
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
    /// ```
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
    ///     user: String,
    /// }
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// let score_board = HighScores(Vec::new());
    /// let assets_dir = "assets/";
    /// let game = Application::build(assets_dir, NullState)?.with_resource(score_board);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn with_resource<R: Resource>(mut self, resource: R) -> Self {
        self.resources.insert(resource);
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
    /// ```no_run
    /// use amethyst::{
    ///     assets::{DefaultLoader, Directory, Handle, Loader},
    ///     prelude::*,
    ///     renderer::{formats::mesh::ObjFormat, Mesh},
    /// };
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// let assets_dir = "assets/";
    /// let game = Application::build(assets_dir, LoadingState)?
    ///     // Register the directory "custom_directory" under the name "resources".
    ///     .with_source("custom_store", Directory::new("custom_directory"))
    ///     .build(DispatcherBuilder::default())?
    ///     .run();
    /// #     Ok(())
    /// # }
    ///
    /// struct LoadingState;
    /// impl SimpleState for LoadingState {
    ///     fn on_start(&mut self, data: StateData<'_, GameData>) {
    ///         let loader = data.resources.get::<DefaultLoader>().unwrap();
    ///         // Load a teapot mesh from the directory that registered above.
    ///         let mesh: Handle<Mesh> = loader.load("teapot.obj");
    ///     }
    /// }
    /// ```
    pub fn with_source<I, O>(self, _name: I, _store: O) -> Self
    where
        I: Into<String>,
        O: Source,
    {
        {
            let _loader = self.resources.get_mut::<DefaultLoader>().unwrap();
            // FIXME Update the source on the loader
            // loader.add_source(name, store);
        }
        self
    }

    /// Registers the default asset store with the loader logic of the Application.
    ///
    /// # Parameters
    ///
    /// - `store`: The asset store being registered.
    ///
    /// # Type Parameters
    ///
    /// - `S`: A `Store` asset loader. Typically this is a [`Directory`](../amethyst_assets/struct.Directory.html).
    ///
    /// # Returns
    ///
    /// This function returns ApplicationBuilder after it has modified it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amethyst::{
    ///     assets::{DefaultLoader, Directory, Handle, Loader, LoaderBundle},
    ///     prelude::*,
    ///     renderer::Mesh,
    /// };
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// #      amethyst::start_logger(Default::default());
    /// let mut dispatcher = DispatcherBuilder::default();
    /// dispatcher.add_bundle(LoaderBundle);
    /// let assets_dir = "assets/";
    /// let game = Application::build(assets_dir, LoadingState)?
    ///     // Register the directory "custom_directory" as default source for the loader.
    ///     .with_default_source(Directory::new("custom_directory"))
    ///     .build(dispatcher)?
    ///     .run();
    /// # Ok(())
    /// # }
    ///
    /// struct LoadingState;
    /// impl SimpleState for LoadingState {
    ///     fn on_start(&mut self, data: StateData<'_, GameData>) {
    ///         let loader = data.resources.get::<DefaultLoader>().unwrap();
    ///         // Load a teapot mesh from the directory that registered above.
    ///         let mesh: Handle<Mesh> = loader.load("teapot.obj");
    ///     }
    /// }
    /// ```
    pub fn with_default_source<O>(self, _store: O) -> Self
    where
        O: Source,
    {
        {
            // let _loader = self.resources.get_mut::<DefaultLoader>().unwrap();
            // FIXME Update the location on the loader
            // loader.set_default_source(store);
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
        self.resources.insert(FrameLimiter::new(strategy, max_fps));
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
        self.resources.insert(FrameLimiter::from_config(config));
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
        self.resources
            .get_mut::<Time>()
            .unwrap()
            .set_fixed_time(duration);
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
    pub fn build<'a, I>(mut self, init: I) -> Result<CoreApplication<'a, T, E, X>, Error>
    where
        S: State<T, E> + 'a,
        I: DataInit<T>,
        E: Clone + Send + Sync + 'static,
        X: EventReader<Event = E> + Default,
    {
        trace!("Entering `ApplicationBuilder::build`");

        #[cfg(feature = "profiler")]
        register_thread_with_profiler();
        #[cfg(feature = "profiler")]
        profile_scope!("new");

        let data = init.build(&mut self.world, &mut self.resources)?;

        let event_reader_id = self
            .resources
            .get_mut::<EventChannel<Event<'static, ()>>>()
            .unwrap()
            .register_reader();

        let mut trans_event_channel = EventChannel::<TransEvent<T, E>>::with_capacity(2);
        let trans_reader_id = trans_event_channel.register_reader();
        self.resources.insert(trans_event_channel);

        let mut reader = X::default();
        reader.setup(&mut self.resources);

        Ok(CoreApplication {
            world: self.world,
            resources: self.resources,
            states: StateMachine::new(self.initial_state),
            reader,
            events: Vec::new(),
            ignore_window_close: self.ignore_window_close,
            data,
            event_reader_id,
            trans_reader_id,
        })
    }
}
