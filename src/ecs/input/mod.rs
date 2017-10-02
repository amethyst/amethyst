//! `amethyst` input rebinding module

pub use self::bundle::InputBundle;
pub use self::system::InputSystem;
pub use input::{Bindings, InputEvent, InputHandler};

pub mod bundle;
mod system;
