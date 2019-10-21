use crate::{BindingTypes, InputEvent, InputHandler};
use amethyst_core::{ecs as specs, legion::*, shrev::EventChannel};
use std::marker::PhantomData;

#[derive(Default)]
pub struct Syncer<T>(PhantomData<T>);
impl<T: BindingTypes> LegionSyncBuilder for Syncer<T> {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    ) {
        state.add_resource_sync::<EventChannel<InputEvent<T>>>();
        state.add_resource_sync::<InputHandler<T>>();
    }
}
