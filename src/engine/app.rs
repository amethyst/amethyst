//! The core engine framework.

use super::state::{State, StateMachine};
use context::timing::{SteadyTime, Stopwatch};
use context::event::EngineEvent;
use context::{Config, Context};
use ecs::{Planner, World, Processor, Priority};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use processors::Renderable;

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    timer: Stopwatch,
    context: Arc<Mutex<Context>>,
}

impl Application {
    /// Creates a new Application with the given initial game state, planner, and config.
    pub fn new<T>(initial_state: T, planner: Planner<Arc<Mutex<Context>>>, config: Config) -> Application
        where T: State + 'static
    {
        let context = Context::new(config);
        let context = Arc::new(Mutex::new(context));
        Application {
            states: StateMachine::new(initial_state, planner),
            timer: Stopwatch::new(),
            context: context,
        }
    }

    /// Build a new Application using builder pattern.
    pub fn build<T>(initial_state: T, config: Config) -> ApplicationBuilder<T>
        where T: State + 'static
    {
        ApplicationBuilder::new(initial_state, config)
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        while self.states.is_running() {
            self.timer.restart();
            self.advance_frame();
            self.timer.stop();
            self.context.lock().unwrap().delta_time = self.timer.elapsed();
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start(self.context.lock().unwrap().deref_mut());
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        // let context = self.context.lock().unwrap().deref_mut();
        let engine_events = self.context.lock().unwrap().poll_engine_events();
        for engine_event in engine_events {
            self.context.lock().unwrap().broadcaster.publish().with::<EngineEvent>(engine_event);
        }
        let events = self.context.lock().unwrap().broadcaster.poll();
        self.states.handle_events(events, self.context.lock().unwrap().deref_mut());
        let fixed_step = self.context.lock().unwrap().fixed_step.clone();
        let last_fixed_update = self.context.lock().unwrap().last_fixed_update.clone();
        if SteadyTime::now() - last_fixed_update > fixed_step {
            self.states.fixed_update(self.context.lock().unwrap().deref_mut());
            // self.systems.fixed_iterate(self.fixed_step);
            self.context.lock().unwrap().last_fixed_update = last_fixed_update + fixed_step;
        }
        self.states.update(self.context.lock().unwrap().deref_mut());
        self.states.run_processors(self.context.clone());
        self.context.lock().unwrap().renderer.submit();
        self.context.lock().unwrap().broadcaster.clean();
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
    config: Config,
    planner: Planner<Arc<Mutex<Context>>>,
}

impl<T> ApplicationBuilder<T>
    where T: State + 'static
{
    pub fn new(initial_state: T, config: Config) -> ApplicationBuilder<T> {
        let mut world = World::new();
        world.register::<Renderable>();
        let planner = Planner::new(world, 1);
        ApplicationBuilder {
            initial_state: initial_state,
            config: config,
            planner: planner,
        }
    }

    pub fn with<P>(mut self,
                   sys: P,
                   name: &str,
                   priority: Priority) -> ApplicationBuilder<T>
        where P: Processor<Arc<Mutex<Context>>> + 'static
    {
        self.planner.add_system::<P>(sys, name, priority);
        self
    }

    pub fn done(self) -> Application {
        Application::new(self.initial_state, self.planner, self.config)
    }
}
