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
use crossbeam_queue::SegQueue;
use derivative::Derivative;
use distill::loader::storage::AssetLoadOp;
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

impl<A> System for AssetProcessorSystem<A>
where
    A: Asset + ProcessableAsset,
{
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new(format!("Asset Processor: {}", A::name()))
                .write_resource::<ProcessingQueue<A::Data>>()
                .write_resource::<AssetStorage<A>>()
                .build(|_, _, (queue, storage), _| {
                    // drain the changed queue
                    while queue.changed.pop().is_some() {}
                    queue.process(storage, ProcessableAsset::process);
                    storage.process_custom_drop(|_| {});
                }),
        )
    }
}

pub(crate) struct LoadNotifier {
    asset_load_op: Option<AssetLoadOp>,
    tracker: Option<Box<dyn Tracker>>,
    load_handle: LoadHandle,
}

impl LoadNotifier {
    pub fn new(
        load_handle: LoadHandle,
        asset_load_op: Option<AssetLoadOp>,
        tracker: Option<Box<dyn Tracker>>,
    ) -> Self {
        Self {
            asset_load_op,
            tracker,
            load_handle,
        }
    }

    /// Signals that this load operation has completed succesfully.
    pub fn complete(self) {
        if let Some(asset_load_op) = self.asset_load_op {
            asset_load_op.complete();
        }
        if let Some(tracker) = self.tracker {
            tracker.success();
        }
    }

    /// Signals that this load operation has completed with an error.
    // FIXME: Make the errors meaningful
    pub fn error(self, error: Error) {
        if let Some(asset_load_op) = self.asset_load_op {
            asset_load_op.error(ProcessingError("ProcessingError".into()));
        }

        if let Some(tracker) = self.tracker {
            tracker.fail(self.load_handle.0, &"", "".to_string(), error);
        }
    }
}

/// Represents asset data processed by `distill` that needs to be loaded by Amethyst.
pub(crate) struct Processed<T> {
    data: Result<T, Error>,
    handle: LoadHandle,
    load_notifier: LoadNotifier,
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
    pub(crate) changed: SegQueue<LoadHandle>,
}

impl<T> Default for ProcessingQueue<T> {
    fn default() -> Self {
        Self {
            processed: Arc::new(SegQueue::new()),
            requeue: Mutex::new(Vec::new()),
            changed: SegQueue::new(),
        }
    }
}

impl<T> ProcessingQueue<T> {
    /// Enqueue asset data for processing
    pub(crate) fn enqueue(
        &self,
        handle: LoadHandle,
        data: T,
        asset_load_op: AssetLoadOp,
        version: u32,
    ) {
        self.enqueue_processed(
            Ok(data),
            handle,
            LoadNotifier::new(handle, Some(asset_load_op), None),
            version,
            false,
        );
    }

    pub(crate) fn enqueue_processed(
        &self,
        data: Result<T, Error>,
        handle: LoadHandle,
        load_notifier: LoadNotifier,
        version: u32,
        commit: bool,
    ) {
        self.processed.push(Processed {
            data,
            handle,
            load_notifier,
            version,
            commit,
        })
    }

    pub(crate) fn enqueue_changed(&self, handle: LoadHandle) {
        self.changed.push(handle);
    }

    pub(crate) fn enqueue_from_data(
        &self,
        handle: LoadHandle,
        data: T,
        tracker: Box<dyn Tracker>,
        version: u32,
    ) {
        self.enqueue_processed(
            Ok(data),
            handle,
            LoadNotifier::new(handle, None, Some(tracker)),
            version,
            true,
        );
    }

    /// Process asset data into assets
    pub fn process<F, A>(&mut self, storage: &mut AssetStorage<A>, mut f: F)
    where
        F: FnMut(T, &mut AssetStorage<A>, &LoadHandle) -> Result<ProcessingState<T, A>, Error>,
    {
        let requeue = self
            .requeue
            .get_mut()
            .expect("The mutex of `requeue` in `AssetStorage` was poisoned");
        while let Some(Processed {
            data,
            handle,
            load_notifier,
            version,
            commit,
        }) = self.processed.pop()
        {
            let f = &mut f;

            let asset = match data.and_then(|d| f(d, storage, &handle)) {
                Ok(ProcessingState::Loaded(x)) => {
                    debug!("{:?} has been loaded successfully", handle);
                    load_notifier.complete();
                    x
                }
                Ok(ProcessingState::Loading(x)) => {
                    requeue.push(Processed {
                        data: Ok(x),
                        handle,
                        load_notifier,
                        version,
                        commit,
                    });
                    continue;
                }
                Err(e) => {
                    load_notifier.error(e);
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

/// Wrapper for string errors.
#[derive(Debug)]
struct ProcessingError(Cow<'static, str>);

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl std::error::Error for ProcessingError {}
