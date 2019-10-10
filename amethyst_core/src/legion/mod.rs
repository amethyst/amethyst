//! TODO: doc
//!

pub mod bundle;
pub mod sync;

pub trait SystemDesc: 'static {
    fn build(
        &self,
        world: &mut legion::world::World,
        resources: &mut legion::resource::Resources,
    ) -> Box<dyn legion::system::Schedulable>;
}

pub use legion::{prelude::*, *};

pub use sync::{LegionSystems as Systems, LegionWorld};

pub trait SystemBundle {
    fn build(
        &self,
        world: &mut legion::world::World,
        resources: &mut legion::resource::Resources,
        systems: &mut Systems,
    ) -> Result<(), amethyst_error::Error>;
}
