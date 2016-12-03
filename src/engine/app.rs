//! The core engine framework.

use super::state::{State, StateMachine};
use renderer;
use renderer::{Light, Pipeline};
use processors::transform::LocalTransform;
use asset_manager::AssetManager;
use gfx_device;
use gfx_device::{GfxDevice, DisplayConfig, Renderable};
use context::timing::Stopwatch;
use ecs::{Planner, World, Processor, Priority, Component};
use std::time::{Duration, Instant};

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    gfx_device: GfxDevice,
    pipeline: Pipeline,
    planner: Planner<()>,
    asset_manager: AssetManager,
    timer: Stopwatch,
    delta_time: Duration,
    fixed_step: Duration,
    last_fixed_update: Instant,
}

pub struct Time {
    pub delta_time: Duration,
    pub fixed_step: Duration,
    pub last_fixed_update: Instant,
}

impl Application {
    /// Creates a new Application with the given initial game state, planner, and display_config.
    pub fn new<T>(initial_state: T,
                  mut planner: Planner<()>,
                  display_config: DisplayConfig)
                  -> Application
        where T: State + 'static
    {
        use gfx_device::camera::{Camera, Projection};
        use gfx_device::screen_dimensions::ScreenDimensions;
        let (gfx_device_inner, gfx_loader, main_target_inner) = gfx_device::video_init(display_config);
        let gfx_device = gfx_device::GfxDevice::new(gfx_device_inner);
        let main_target = gfx_device::MainTarget::new(main_target_inner);
        // FIXME Remove all platform specific code from here!
        let mut pipeline = Pipeline::new();
        match main_target.main_target_inner {
            gfx_device::MainTargetInner::OpenGL {
                ref main_color,
                ref main_depth,
            } => {
                pipeline.targets.insert("main".into(),
                                     Box::new(renderer::target::ColorBuffer {
                                         color: main_color.clone(),
                                         output_depth: main_depth.clone(),
                                     }));

                // let (w, h) = window.get_inner_size().unwrap();
                // pipeline.targets.insert("gbuffer".into(),
                //                      Box::new(renderer::target::GeometryBuffer::new(&mut factory, (w as u16, h as u16))));
            },
            #[cfg(windows)]
            gfx_device::MainTargetInner::Direct3D {  } =>  unimplemented!(),
            gfx_device::MainTargetInner::Null => (),
        };
        let mut asset_manager = AssetManager::new();
        asset_manager.add_loader::<gfx_device::gfx_loader::GfxLoader>(gfx_loader);
        {
            let mut world = planner.mut_world();
            let time = Time {
                delta_time: Duration::new(0, 0),
                fixed_step: Duration::new(0, 16666666),
                last_fixed_update: Instant::now(),
            };
            if let Some ((w, h)) = gfx_device.get_dimensions() {
                let dimensions = ScreenDimensions::new(w, h);
                let projection = Projection::Perspective {
                    fov: 90.0,
                    aspect_ratio: dimensions.aspect_ratio,
                    near: 0.1,
                    far: 100.0,
                };
                let eye = [0.0, 0.0, 0.0];
                let target = [1.0, 0.0, 0.0];
                let up = [0.0, 1.0, 0.0];
                let camera = Camera::new(projection, eye, target, up);
                world.add_resource::<ScreenDimensions>(dimensions);
                world.add_resource::<Camera>(camera);
            }
            world.add_resource::<Time>(time);
            world.register::<Renderable>();
            world.register::<LocalTransform>();
            world.register::<Light>();
        }
        Application {
            states: StateMachine::new(initial_state),
            gfx_device: gfx_device,
            planner: planner,
            pipeline: pipeline,
            asset_manager: asset_manager,
            timer: Stopwatch::new(),
            delta_time: Duration::new(0, 0),
            fixed_step: Duration::new(0, 16666666),
            last_fixed_update: Instant::now(),
        }
    }

    /// Build a new Application using builder pattern.
    pub fn build<T>(initial_state: T, display_config: DisplayConfig) -> ApplicationBuilder<T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state, display_config)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.delta_time = self.timer.elapsed();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(self.planner.mut_world(), &mut self.asset_manager, &mut self.pipeline);
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        {
            let events = self.gfx_device.poll_events();

            self.states.handle_events(events.as_ref(), self.planner.mut_world(), &mut self.asset_manager, &mut self.pipeline);

            let fixed_step = self.fixed_step;
            let last_fixed_update = self.last_fixed_update;

            if last_fixed_update.elapsed() >= fixed_step {
                self.states.fixed_update(self.planner.mut_world(), &mut self.asset_manager, &mut self.pipeline);
                self.last_fixed_update += fixed_step;
            }

            self.states.update(self.planner.mut_world(), &mut self.asset_manager, &mut self.pipeline);
        }
        self.planner.dispatch(());
        self.planner.wait();
        let world = self.planner.mut_world();
        {
            let mut time = world.write_resource::<Time>();
            time.delta_time = self.delta_time;
            time.fixed_step = self.fixed_step;
            time.last_fixed_update = self.last_fixed_update;
        }
        self.gfx_device.render_world(world, &self.pipeline);
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder
    }
}

/// Helper builder for Applications.
pub struct ApplicationBuilder<T>
    where T: State + 'static
{
    initial_state: T,
    display_config: DisplayConfig,
    planner: Planner<()>,
}

impl<T> ApplicationBuilder<T>
    where T: State + 'static
{
    pub fn new(initial_state: T, display_config: DisplayConfig) -> ApplicationBuilder<T> {
        let world = World::new();
        ApplicationBuilder {
            initial_state: initial_state,
            display_config: display_config,
            planner: Planner::new(world, 1),
        }
    }

    pub fn register<C>(mut self) -> ApplicationBuilder<T>
        where C: Component
    {
        {
            let world = &mut self.planner.mut_world();
            world.register::<C>();
        }
        self
    }

    pub fn with<P>(mut self, pro: P, name: &str, pri: Priority) -> ApplicationBuilder<T>
        where P: Processor<()> + 'static
    {
        self.planner.add_system::<P>(pro, name, pri);
        self
    }

    pub fn done(self) -> Application {
        Application::new(self.initial_state, self.planner, self.display_config)
    }
}
