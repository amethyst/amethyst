use std::{
    borrow::Cow,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use amethyst_core::{
    dispatcher::System,
    ecs::{systems::ParallelRunnable, SystemBuilder},
};
use amethyst_error::Error;
use atelier_assets::loader::storage::AssetLoadOp;
use crossbeam_queue::SegQueue;
use derivative::Derivative;
use log::debug;

use crate::{
    asset::{Asset, ProcessableAsset},
    loader::LoadHandle,
    progress::Tracker,
    storage::AssetStorage,
};

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct AssetProcessorSystem<A> {
    _marker: PhantomData<A>,
}

impl<A> System<'_> for AssetProcessorSystem<A>
where
    A: Asset + ProcessableAsset,
{
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new(format!("Asset Processor: {}", A::name()))
                .write_resource::<ProcessingQueue<A::Data>>()
                .write_resource::<AssetStorage<A>>()
                .build(|_, _, (queue, storage), _| {
                    queue.process(storage, ProcessableAsset::process);
                    storage.process_custom_drop(|_| {});
                }),
        )
    }
}

/// Represents asset data processed by `atelier-assets` that needs to be loaded by Amethyst.
pub(crate) struct Processed<T> {
    data: Result<T, Error>,
    handle: LoadHandle,
    tracker: Option<Box<dyn Tracker>>,
    load_op: Option<AssetLoadOp>,
    version: u32,
    commit: bool,
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
    pub(crate) fn enqueue(&self, handle: LoadHandle, data: T, load_op: AssetLoadOp, version: u32) {
        self.enqueue_processed(Ok(data), handle, None, Some(load_op), version, false);
    }

    fn enqueue_processed(
        &self,
        data: Result<T, Error>,
        handle: LoadHandle,
        tracker: Option<Box<dyn Tracker>>,
        load_op: Option<AssetLoadOp>,
        version: u32,
        commit: bool,
    ) {
        self.processed.push(Processed {
            data,
            handle,
            tracker,
            load_op,
            version,
            commit,
        })
    }

    pub(crate) fn enqueue_from_data(
        &self,
        handle: LoadHandle,
        data: T,
        tracker: Box<dyn Tracker>,
        version: u32,
    ) {
        self.enqueue_processed(Ok(data), handle, Some(tracker), None, version, true);
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
                let Processed {
                    data,
                    handle,
                    tracker,
                    load_op,
                    version,
                    commit,
                } = processed;

                let asset = match data.and_then(|d| f(d)) {
                    Ok(ProcessingState::Loaded(x)) => {
                        debug!(
                            "Asset (handle id: {:?}) has been loaded successfully",
                            handle,
                        );

                        if let Some(tracker) = tracker {
                            tracker.success();
                        }
                        if let Some(op) = load_op {
                            op.complete();
                        }
                        x
                    }
                    Ok(ProcessingState::Loading(x)) => {
                        requeue.push(Processed {
                            data: Ok(x),
                            handle,
                            tracker,
                            load_op,
                            version,
                            commit,
                        });
                        continue;
                    }
                    Err(e) => {
                        if let Some(tracker) = tracker {
                            tracker.fail(handle.0, &"", "".to_string(), e);
                        }
                        if let Some(op) = load_op {
                            op.error(ProcessingError("ProcessingError".into()));
                        }
                        continue;
                    }
                };
                storage.update_asset(handle, asset, version);
                if commit {
                    storage.commit_asset(handle, version);
                }
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
