use atelier_loader::storage::AssetLoadOp;
use crossbeam_queue::SegQueue;
use std::borrow::Cow;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use amethyst_core::ecs::{systems::ParallelRunnable, DispatcherBuilder, SystemBuilder};
use amethyst_error::Error;

use crate::{
    asset::{Asset, ProcessableAsset},
    // error::Error,
    loader_new::LoadHandle,
    progress::Tracker,
    storage_new::AssetStorage,
};

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
// pub struct Processor<A> {
//     marker: PhantomData<A>,
// }

// impl<A> Default for Processor<A> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl<A> Processor<A> {
//     /// Creates a new asset processor for assets of type `A`.
//     pub fn new() -> Self {
//         Processor {
//             marker: PhantomData,
//         }
//     }
// }

// impl<'a, A> System<'a> for Processor<A>
// where
//     A: crate::asset::Asset,
//     A::Data: Into<Result<ProcessingState<A::Data, A>, Error>>,
// {
//     type SystemData = (
//         Write<'a, ProcessingQueue<A::Data>>,
//         Write<'a, AssetStorage<A>>,
//     );

//     fn run(&mut self, (mut queue, mut storage): Self::SystemData) {
//     }
// }

pub struct DefaultProcessor<A> {
    marker: PhantomData<A>,
}

impl<A> AddToDispatcher for DefaultProcessor<A>
where
    A: Asset + ProcessableAsset,
{
    fn add_to_dipatcher(dispatcher_builder: &mut DispatcherBuilder) {
        dispatcher_builder.add_system(build_default_asset_processer_system::<A>());
    }
}

pub trait AddToDispatcher {
    fn add_to_dipatcher(dispatcher_builder: &mut DispatcherBuilder);
}

pub fn build_default_asset_processer_system<A>() -> impl ParallelRunnable
where
    A: Asset + ProcessableAsset,
{
    SystemBuilder::new(format!("Asset Processor {}", A::name()))
        .write_resource::<ProcessingQueue<A::Data>>()
        .write_resource::<AssetStorage<A>>()
        .build(|_, _, (queue, storage), _| {
            queue.process(storage, ProcessableAsset::process);
            storage.process_custom_drop(|_| {});
        })
}

/// Represents asset data processed by `atelier-assets` that needs to be loaded by Amethyst.
pub(crate) struct Processed<T> {
    data: Result<T, Error>,
    handle: LoadHandle,
    tracker: Option<Box<dyn Tracker>>,
    load_op: Option<AssetLoadOp>,
    version: u32,
}

/// Returned by processor systems, describes the loading state of the asset.
pub enum ProcessingState<D, A> {
    /// Asset is not fully loaded yet, need to wait longer
    Loading(D),
    /// Asset have finished loading, can now be inserted into storage and tracker notified
    Loaded(A),
}

/// Queue of processed asset data, to be loaded by Amethyst.
///
/// # Type Parameters
///
/// `T`: Asset data type.
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
    pub(crate) fn enqueue(
        &self,
        handle: LoadHandle,
        data: T,
        load_op: Option<AssetLoadOp>,
        version: u32,
    ) {
        self.enqueue_with_tracker(handle, data, None, load_op, version)
    }
    pub(crate) fn enqueue_with_tracker(
        &self,
        handle: LoadHandle,
        data: T,
        tracker: Option<Box<dyn Tracker>>,
        load_op: Option<AssetLoadOp>,
        version: u32,
    ) {
        self.processed.push(Processed {
            data: Ok(data),
            handle,
            tracker,
            load_op,
            version,
        })
    }
    /// Process asset data into assets
    pub fn process<F, A>(&mut self, storage: &mut AssetStorage<A>, mut f: F)
    where
        F: FnMut(T) -> Result<ProcessingState<T, A>, Error>,
    {
        {
            let requeue = self
                .requeue
                .get_mut()
                .expect("The mutex of `requeue` in `AssetStorage` was poisoned");
            while let Ok(processed) = self.processed.pop() {
                let f = &mut f;
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

                                if let Some(tracker) = tracker {
                                    tracker.success();
                                }
                                if let Some(op) = load_op {
                                    op.complete();
                                }
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
                                if let Some(tracker) = tracker {
                                    // FIXME
                                    tracker.fail(handle.0, &"", "".to_string(), Error::from_string(""));
                                }
                                if let Some(op) = load_op {
                                    // FIXME
                                    op.error(ProcessingError("ProcessingError".into()));
                                }
                                continue;
                            }
                        };
                        storage.update_asset(handle, asset, version);
                    }
                };
            }

            for p in requeue.drain(..) {
                self.processed.push(p);
            }
        }
    }
}

/// Wrapper for string errors.
#[derive(Debug)]
struct ProcessingError(Cow<'static, str>);

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl std::error::Error for ProcessingError {}
// struct ProcessingError;

// impl std::error::Error for ProcessingError {}
// impl std::fmt::Display for ProcessingError {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Display::fmt(&self.inner.error, fmt)
//     }
// }

// impl std::fmt::Debug for ProcessingError {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         fmt.debug_struct("Error")
//             .field("inner", &self.inner)
//             .finish()
//     }
// }
