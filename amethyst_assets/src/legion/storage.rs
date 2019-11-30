use crate::{
    asset::{Asset, ProcessableAsset},
    storage::AssetStorage,
};
use amethyst_core::{legion::*, ArcThreadPool, Time};
use derivative::Derivative;
use std::marker::PhantomData;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
pub fn build_asset_processor<A: Asset + ProcessableAsset>(
    world: &mut World,
) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new(std::any::type_name::<A>())
        .write_resource::<AssetStorage<A>>()
        .read_resource::<ArcThreadPool>()
        .read_resource::<Time>()
        //          .read_resource::<HotReloadStrategy>() TODO: we should allow options
        .build(move |commands, world, (storage, pool, time), _| {
            #[cfg(feature = "profiler")]
            profile_scope!("processor_system");

            storage.process(
                ProcessableAsset::process,
                time.frame_number(),
                &**pool,
                None, //strategy.as_ref().map(Deref::deref)
            );
        })
}
