//! Structs and enums holding graphics resources like `gfx::Device`,
//! `gfx::Factory`, `glutin::Window`, etc.)

mod display_config;
mod gfx_device;
mod video_init;

pub mod gfx_types;

pub use self::display_config::DisplayConfig;
pub use self::gfx_device::*;
pub use self::video_init::video_init;
