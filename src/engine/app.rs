//! The core engine framework.

use super::state::{State, StateMachine};
use context::timing::{SteadyTime, Stopwatch};
use context::event::EngineEvent;
use context::{Config, Context};
use ecs::Planner;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    timer: Stopwatch,
    context: Arc<Mutex<Context>>,
}

impl Application {
    /// Creates a new Application with the given initial game state and a given `Context`.
    pub fn new<T: 'static>(initial_state: T, planner: Planner<Arc<Mutex<Context>>>, config: Config) -> Application
        where T: State
    {
        let context = Context::new(config);
        let context = Arc::new(Mutex::new(context));
        Application {
            states: StateMachine::new(initial_state, planner),
            timer: Stopwatch::new(),
            context: context,
        }
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
        while SteadyTime::now() - self.context.lock().unwrap().last_fixed_update > fixed_step {
            self.states.fixed_update(self.context.lock().unwrap().deref_mut());
            // self.systems.fixed_iterate(self.fixed_step);
            self.context.lock().unwrap().last_fixed_update = last_fixed_update + fixed_step;
        }
        self.states.update(self.context.lock().unwrap().deref_mut());
        self.states.run_processors(self.context.clone());
        self.context.lock().unwrap().broadcaster.clean();
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        // Placeholder
    }
}
