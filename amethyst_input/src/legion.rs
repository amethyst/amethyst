use crate::{BindingTypes, InputEvent, InputHandler};
use amethyst_core::{ecs as specs, legion::*, shrev::EventChannel};
use derivative::Derivative;
use std::marker::PhantomData;

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct Syncer<T>(PhantomData<T>);
impl<T: BindingTypes> LegionSyncBuilder for Syncer<T> {
    fn prepare(
        &mut self,
        _: &mut specs::World,
        state: &mut LegionState,
        _: &mut DispatcherBuilder<'_>,
    ) {
        state.add_resource_sync::<EventChannel<InputEvent<T>>>();
        state.add_resource_sync::<InputHandler<T>>();
    }
}
