pub mod bundle;
pub mod system;

pub use bundle::*;
pub use system::*;

use amethyst_core::{ecs as specs, legion::*, shrev::EventChannel};
#[derive(Debug, Default)]
pub struct Syncer;
impl LegionSyncBuilder for Syncer {
    fn prepare(
        &mut self,
        _: &mut specs::World,
        state: &mut LegionState,
        _: &mut DispatcherBuilder<'_>,
    ) {
        state.add_resource_sync::<EventChannel<winit::Event>>();
        state.add_resource_sync::<crate::ScreenDimensions>();
        state.add_resource_sync::<crate::Window>();
    }
}
