//! Resources that can be added to `ecs::World`.
//!
//! `Camera`, `ScreenDimensions`, and `Time` are added by default and
//! automatically updated every frame by `Application`.

pub mod camera;
pub mod screen_dimensions;
pub mod time;
pub mod input;
pub mod broadcaster;
pub mod clear_color;

pub use self::broadcaster::Broadcaster;
pub use self::camera::{Camera, Projection};
pub use self::input::InputHandler;
pub use self::screen_dimensions::ScreenDimensions;
pub use self::time::Time;
pub use self::clear_color::ClearColor;
