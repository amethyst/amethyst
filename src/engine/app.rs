use engine::state::{State, StateMachine};
use engine::timing::{Duration, SteadyTime, Stopwatch};

/// Times a function and returns the elapsed time.
macro_rules! benchmark {
    ($function:expr) => {{
        let mut timer = Stopwatch::new();
        timer.restart();
        $function;
        timer.stop();
        timer.elapsed()
    }}
}

/// User-friendly facade for building games. Manages main loop.
pub struct Application {
    states: StateMachine,
    last_fixed_update: SteadyTime,
    fixed_step: Duration,
    delta_time: Duration,
}

impl Application {
    pub fn new<T: 'static>(initial_state: T) -> Application
        where T: State
    {
        Application {
            states: StateMachine::new(initial_state),
            last_fixed_update: SteadyTime::now(),
            fixed_step: Duration::microseconds(16666),
            delta_time: Duration::zero(),
        }
    }

    /// Starts the application and manages the game loop.
    pub fn run(&mut self) {
        self.initialize();

        loop {
            self.delta_time = benchmark!(self.advance_frame());
        }

        self.shutdown();
    }

    /// Sets up the application.
    fn initialize(&mut self) {
        self.states.start();
    }

    /// Advances the game world by one tick.
    fn advance_frame(&mut self) {
        // self.states.handle_events(&self.event_queue.poll());

        while SteadyTime::now() - self.last_fixed_update > self.fixed_step {
            self.states.fixed_update(self.fixed_step);
            // self.systems.fixed_iterate(self.fixed_step);
            self.last_fixed_update = self.last_fixed_update + self.fixed_step;
        }

        self.states.update(self.delta_time);
        // self.systems.iterate(self.delta_time);
    }

    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {
        self.states.stop()
    }
}
