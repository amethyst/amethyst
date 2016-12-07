extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

mod gfx_device_inner;
mod gfx_device;
mod main_target;
mod video_init;
mod display_config;

pub mod gfx_loader;

pub use self::display_config::DisplayConfig;
pub use self::video_init::video_init;
pub use self::main_target::*;
pub use self::gfx_device::*;
