//! High level rendering engine with multiple backends.

pub mod backend;
pub mod frontend;
pub mod ir;
pub mod types;

pub use self::backend::Backend;
pub use self::frontend::{Frame, Frontend, Renderable};
