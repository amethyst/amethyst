//! TODO

#[doc(inline)]
pub mod map;
pub use map::*;
#[doc(inline)]
pub mod pass;
pub use pass::*;

pub use crate::{CoordinateEncoder, MortonEncoder2D};
use amethyst_core::{
    ecs as specs,
    legion::{DispatcherBuilder, LegionState, LegionSyncBuilder},
};
use std::marker::PhantomData;

#[derive(Default)]
/// Temporary migration syncer for tilemaps
pub struct Syncer<T, E = MortonEncoder2D>(PhantomData<(T, E)>);
impl<T, E> LegionSyncBuilder for Syncer<T, E>
where
    T: Tile,
    E: CoordinateEncoder,
{
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder<'_>,
    ) {
        state.add_component_sync::<TileMap<T, E>>();
    }
}
