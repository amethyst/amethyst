//! Resources that can be added to `ecs::World`.
//!
//! `Camera`, `ScreenDimensions`, and `Time` are added by default and
//! automatically updated every frame by `Application`.

mod camera;
mod screen_dimensions;
mod time;
mod input;
mod broadcaster;

pub use self::broadcaster::Broadcaster;
pub use self::camera::{Camera, Projection};
pub use self::input::{Axis, Button, InputHandler, ButtonIterator};
pub use self::screen_dimensions::ScreenDimensions;
pub use self::time::Time;
