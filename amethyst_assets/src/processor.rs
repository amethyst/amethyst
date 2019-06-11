use std::{
    marker::PhantomData,
    sync::{
        Arc, Mutex,
    },
};

use crossbeam::queue::SegQueue;
use atelier_loader::AssetLoadOp;

use amethyst_core::ecs::{
    prelude::{System, Write},
};

use crate::{
    asset::{Asset},
    error::{Error},
    loader_new::LoadHandle,
    storage_new::AssetStorage,
    progress::Tracker,
};

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
pub struct Processor<A> {
    marker: PhantomData<A>,
}

impl<A> Processor<A> {
    /// Creates a new asset processor for
    /// assets of type `A`.
    pub fn new() -> Self {
        Processor {
            marker: PhantomData,
        }
    }
}

impl<'a, A> System<'a> for Processor<A>
where
    A: Asset,
    A::Data: Into<Result<ProcessingState<A::Data, A>, Error>>,
{
    type SystemData = (
        Write<'a, ProcessingQueue<A::Data>>,
        Write<'a, AssetStorage<A>>,
    );

    fn run(&mut self, (mut queue, mut storage): Self::SystemData) {
        queue.process(&mut storage, Into::into);
        storage.process_custom_drop(|_| {});
    }
}

pub(crate) struct Processed<T> {
    data: Result<T, Error>,
    handle: LoadHandle,
    tracker: Option<Box<dyn Tracker>>,
    load_op: AssetLoadOp,
    version: u32,
}

/// Returned by processor systems, describes the loading state of the asset.
pub enum ProcessingState<D, A> {
    /// Asset is not fully loaded yet, need to wait longer
    Loading(D),
    /// Asset have finished loading, can now be inserted into storage and tracker notified
    Loaded(A),
}

pub struct ProcessingQueue<T> {
    pub(crate) processed: Arc<SegQueue<Processed<T>>>,
    requeue: Mutex<Vec<Processed<T>>>,
}
impl<T> Default for ProcessingQueue<T> {
    fn default() -> Self {
        Self {
            processed: Arc::new(SegQueue::new()),
            requeue: Mutex::new(Vec::new()),
        }
    }
}

impl<T> ProcessingQueue<T> {
    /// Enqueue asset data for processing
    pub(crate) fn enqueue(&self, handle: LoadHandle, data: T, load_op: AssetLoadOp, version: u32) {
        self.processed.push(Processed {
            data: Ok(data),
            handle,
            tracker: None,
            load_op,
            version,
        })
    }
    /// Process asset data into assets
    pub fn process<F, A: Asset>(&mut self, storage: &mut AssetStorage<A>, mut f: F)
    where
        F: FnMut(T) -> Result<ProcessingState<T, A>, Error>,
    {
        {
            let requeue = self
                .requeue
                .get_mut()
                .expect("The mutex of `requeue` in `AssetStorage` was poisoned");
            while let Some(processed) = self.processed.try_pop() {
                let f = &mut f;
                log::info!("loaded {}", A::name());
                match processed {
                    Processed {
                        data,
                        handle,
                        tracker,
                        load_op,
                        version,
                    } => {
                        let asset = match data
                            .and_then(|d| f(d))
                            // .chain_err(|| ErrorKind::Asset(name.clone()))
                        {
                            Ok(ProcessingState::Loaded(x)) => {
                                // debug!(
                                //         "{:?}: Asset {:?} (handle id: {:?}) has been loaded successfully",
                                //         A::name(),
                                //         name,
                                //         handle,
                                //     );
                                // TODO do this in loader?
                                // // Add a warning if a handle is unique (i.e. asset does not
                                // // need to be loaded as it is not used by anything)
                                // // https://github.com/amethyst/amethyst/issues/628
                                // if handle.is_unique() {
                                //     warn!(
                                //         "Loading unnecessary asset. Handle {} is unique ",
                                //         handle.id()
                                //     );
                                //     if let Some(tracker) = tracker {
                                //         tracker.fail(
                                //             handle.id(),
                                //             A::name(),
                                //             name,
                                //             Error::from_kind(ErrorKind::UnusedHandle),
                                //         );
                                //     }
                                // } else if let Some(tracker) = tracker {
                                //     tracker.success();
                                // }
                                
                                load_op.complete();
                                x
                            }
                            Ok(ProcessingState::Loading(x)) => {
                                // debug!(
                                //         "{:?}: Asset {:?} (handle id: {:?}) is not complete, readding to queue",
                                //         A::name(),
                                //         name,
                                //         handle,
                                //     );
                                requeue.push(Processed {
                                    data: Ok(x),
                                    handle,
                                    tracker,
                                    load_op,
                                    version,
                                });
                                continue;
                            }
                            Err(e) => {
                                // error!(
                                //     "{:?}: Asset {:?} (handle id: {:?}) could not be loaded: {}",
                                //     A::name(),
                                //     name,
                                //     handle,
                                //     e,
                                // );
                                // if let Some(tracker) = tracker {
                                //     tracker.fail(handle, A::name(), name, e);
                                // }
                                load_op.error(e);

                                continue;
                            }
                        };
                        storage.update_asset(&handle, asset, version);
                    }
                };
            }

            for p in requeue.drain(..) {
                self.processed.push(p);
            }
        }
    }
}
