//! The core engine framework.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::ThreadPool;
#[cfg(feature="profiler")]
use thread_profiler::{register_thread_with_profiler, write_profile};
use num_cpus;

use asset_manager::AssetManager;
use ecs::{Component, Dispatcher, DispatcherBuilder, System, World};
use ecs::components::{LocalTransform, Transform, Child, Init, Renderable};
use ecs::resources::Time;
use engine::state::{State, StateMachine};
use engine::timing::Stopwatch;
use gfx_device;
use gfx_device::{DisplayConfig, GfxDevice, gfx_types};
use renderer::{AmbientLight, DirectionalLight, Pipeline, PointLight, target};

/// User-friendly facade for building games. Manages main loop.
pub struct Application<'a, 'b> {
    // Graphics and asset management structs.
    // TODO: Refactor so `pipe` and `gfx_device` are moved into the renderer.
    assets: AssetManager,
    gfx_device: GfxDevice,
    pipe: Pipeline,
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
                  dispatcher: Dispatcher<'a, 'b>,
                  mut world: World,
                  cfg: DisplayConfig)
                  -> Application<'a, 'b>
        where T: State + 'static
    {
        use ecs::resources::{Camera, Projection, ScreenDimensions};

        #[cfg(feature="profiler")]
        register_thread_with_profiler("Main".into());
        #[cfg(feature="profiler")]
        profile_scope!("video_init");
        let (device, mut factory, main_target) = gfx_device::video_init(&cfg);
        let mut pipe = Pipeline::new();
        pipe.targets
            .insert("main".into(),
                    Box::new(target::ColorBuffer {
                                 color: main_target.color.clone(),
                                 output_depth: main_target.depth.clone(),
                             }));

        let (w, h) = device.get_dimensions().unwrap();
        let geom_buf = target::GeometryBuffer::new(&mut factory, (w as u16, h as u16));
        pipe.targets.insert("gbuffer".into(), Box::new(geom_buf));

        let mut assets = AssetManager::new();
        assets.add_loader::<gfx_types::Factory>(factory);

        {
            let time = Time {
                delta_time: Duration::new(0, 0),
                fixed_step: Duration::new(0, 16666666),
                last_fixed_update: Instant::now(),
            };
            if let Some((w, h)) = device.get_dimensions() {
                let dim = ScreenDimensions::new(w, h);
                let proj = Projection::Perspective {
                    fov: 90.0,
                    aspect_ratio: dim.aspect_ratio,
                    near: 0.1,
                    far: 100.0,
                };
                let eye = [0.0, 0.0, 0.0];
                let target = [1.0, 0.0, 0.0];
                let up = [0.0, 1.0, 0.0];
                let camera = Camera::new(proj, eye, target, up);
                world.add_resource::<ScreenDimensions>(dim);
                world.add_resource::<Camera>(camera);
            }

            world.add_resource::<AmbientLight>(AmbientLight::default());
            world.add_resource::<Time>(time);
            world.register::<Child>();
            world.register::<DirectionalLight>();
            world.register::<Init>();
            world.register::<LocalTransform>();
            world.register::<PointLight>();
            world.register::<Renderable>();
            world.register::<Transform>();
        }

        Application {
            assets: assets,
            states: StateMachine::new(initial_state),
            gfx_device: device,
            pipe: pipe,
            dispatcher: dispatcher,
            world: world,
            timer: Stopwatch::new(),
            delta_time: Duration::new(0, 0),
            fixed_step: Duration::new(0, 16666666),
            last_fixed_update: Instant::now(),
        }
    }

    /// Builds a new application using builder pattern.
    pub fn build<T>(initial_state: T, cfg: DisplayConfig) -> ApplicationBuilder<'a, 'b, T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state, cfg)
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
        let pipe = &mut self.pipe;
        self.states.start(world, assets, pipe);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        use ecs::resources::ScreenDimensions;
        {
            #[cfg(feature="profiler")]
            profile_scope!("handle_events");
            let events = self.gfx_device.poll_events();
            let world = &mut self.world;
            let assets = &mut self.assets;
            let pipe = &mut self.pipe;

            self.states
                .handle_events(events.as_ref(), world, assets, pipe);

            #[cfg(feature="profiler")]
            profile_scope!("fixed_update");
            if self.last_fixed_update.elapsed() >= self.fixed_step {
                self.states.fixed_update(world, assets, pipe);
                self.last_fixed_update += self.fixed_step;
            }

            #[cfg(feature="profiler")]
            profile_scope!("update");
            self.states.update(world, assets, pipe);
        }

        #[cfg(feature="profiler")]
        profile_scope!("dispatch");
        self.dispatcher.dispatch(&mut self.world.res);

        #[cfg(feature="profiler")]
        profile_scope!("render_world");
        {
            let world = &mut self.world;
            if let Some((w, h)) = self.gfx_device.get_dimensions() {
                let mut dim = world.write_resource::<ScreenDimensions>();
                dim.update(w, h);
            }

            {
                let mut time = world.write_resource::<Time>();
                time.delta_time = self.delta_time;
                time.fixed_step = self.fixed_step;
                time.last_fixed_update = self.last_fixed_update;
            }

            let pipe = &mut self.pipe;
            self.gfx_device.render_world(world, pipe);
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
pub struct ApplicationBuilder<'a, 'b, T>
    where T: State + 'static
{
    config: DisplayConfig,
    initial_state: T,
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
    world: World,
}

impl<'a, 'b, T> ApplicationBuilder<'a, 'b, T>
    where T: State + 'static
{
    /// Creates a new ApplicationBuilder with the given initial game state and
    /// display configuration.
    pub fn new(initial_state: T, cfg: DisplayConfig) -> ApplicationBuilder<'a, 'b, T> {
        use rayon::Configuration;

        let pool = Arc::new(ThreadPool::new(Configuration::new().num_threads(num_cpus::get()))
                                .expect("Failed to create rayon::ThreadPool"));

        ApplicationBuilder {
            config: cfg,
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
                         self.world,
                         self.config)
    }
}
