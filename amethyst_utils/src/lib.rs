extern crate amethyst_core;
#[macro_use]
extern crate log;
extern crate shred;
extern crate specs;
extern crate winit;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub mod fps_counter;
pub mod circular_buffer;
