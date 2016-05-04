extern crate time;
extern crate specs;

mod processor;
mod sim;

pub use specs::*;
pub use self::processor::{Processor, ProcessorResult};
pub use self::sim::{Simulation, SimBuilder};
