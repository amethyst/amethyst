pub mod bundle;
pub mod system;

pub use bundle::*;
pub use system::*;

use amethyst_core::{ecs as specs, legion::*, shrev::EventChannel};
use std::marker::PhantomData;

#[derive(Default)]
pub struct Syncer;
impl LegionSyncBuilder for Syncer {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    ) {
        state.add_resource_sync::<EventChannel<winit::Event>>();
        state.add_resource_sync::<crate::ScreenDimensions>();
        state.add_resource_sync::<crate::Window>();
    }
}
