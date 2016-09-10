//! Computes the next state.

use super::{World, Processor};

use specs::Planner;
use time::Duration;

pub struct Simulation {
    planner: Planner<Duration>,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new(world: World, num_threads: usize) -> Simulation {
        Simulation { planner: Planner::new(world, num_threads) }
    }

    /// Creates an initialized simulation using the [builder pattern][bp].
    ///
    /// [bp]: https://doc.rust-lang.org/book/method-syntax.html#builder-pattern
    pub fn build(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder::new(world, num_threads)
    }

    /// Adds a new processor to the simulation.
    pub fn add_processor<P>(&mut self, pro: P, name: &str, priority: i32)
        where P: Processor<Duration> + 'static
    {
        self.planner.add_system(pro, name, priority);
    }

    /// Get a mutable reference to the world.
    pub fn mut_world(&mut self) -> &mut World {
        self.planner.mut_world()
    }

    /// Computes the next state of the world using the given processors.
    pub fn step(&mut self, dt: Duration) {
        self.planner.dispatch(dt);
    }
}

impl Drop for Simulation {
    fn drop(&mut self) {
        self.planner.wait();
    }
}

/// Consuming builder for easily constructing a new simulations.
pub struct SimBuilder {
    sim: Simulation,
}

impl SimBuilder {
    /// Starts building a new simulation.
    pub fn new(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder { sim: Simulation::new(world, num_threads) }
    }

    /// Add a given processor to the simulation.
    pub fn with<P>(mut self, pro: P, name: &str, priority: i32) -> SimBuilder
        where P: Processor<Duration> + 'static
    {
        self.sim.add_processor(pro, name, priority);
        self
    }

    /// Returns the newly-built simulation or a list of any errors the
    /// processors may have encountered.
    pub fn done(self) -> Simulation {
        self.sim
    }
}
