//! TODO: doc
//!

pub mod bundle;
pub mod sync;
pub trait LegionSystemDesc: 'static {
    fn build(&self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable>;
}

pub use legion::{prelude::*, *};
