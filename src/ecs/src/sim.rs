//! Computes the next state.

use time::Duration;

use super::{World, Planner};
use processor::{Processor, ProcessorResult};

pub struct Simulation {
    planner: Planner,
    procs: Vec<Box<Processor>>,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new(world: World, num_threads: usize) -> Simulation {
        Simulation {
            planner: Planner::new(world, num_threads),
            procs: Vec::new(),
        }
    }

    /// Creates an initialized simulation using the [builder pattern][bp].
    ///
    /// [bp]: https://doc.rust-lang.org/book/method-syntax.html#builder-pattern
    pub fn build(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder::new(world, num_threads)
    }

    /// Adds a new processor to the simulation.
    pub fn add_processor<T: Processor + 'static>(&mut self, p: T) -> ProcessorResult {
        self.procs.push(Box::new(p));
        Ok(())
    }

    /// Get a reference to the world.
    pub fn world(&self) -> &World {
        &self.planner.world
    }

    /// Computes the next state of the world using the given processors.
    pub fn step(&mut self, dt: Duration) {
        for p in self.procs.iter_mut() {
            p.process(&mut self.planner, dt);
        }
        self.planner.wait();
    }
}

/// Consuming builder for easily constructing a new simulations.
pub struct SimBuilder {
    errors: Vec<String>,
    sim: Simulation,
}

impl SimBuilder {
    /// Starts building a new simulation.
    pub fn new(world: World, num_threads: usize) -> SimBuilder {
        SimBuilder {
            errors: Vec::new(),
            sim: Simulation::new(world, num_threads),
        }
    }

    /// Add a given processor to the simulation.
    pub fn with<T: Processor + 'static>(mut self, p: T) -> SimBuilder {
        let r = self.sim.add_processor(p);
        if let Err(e) = r {
            self.errors.push(e);
        }
        self
    }

    /// Returns the newly-built simulation or a list of any errors the
    /// processors may have encountered.
    pub fn done(self) -> Result<Simulation, Vec<String>> {
        if self.errors.is_empty() {
            Ok(self.sim)
        } else {
            Err(self.errors)
        }
    }
}
