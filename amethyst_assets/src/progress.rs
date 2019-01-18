use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use log::error;
use parking_lot::Mutex;

use crate::Error;

/// Completion status, returned by `ProgressCounter::complete`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    /// Loading is complete
    Complete,
    /// Some asset loads have failed
    Failed,
    /// Still loading assets
    Loading,
}

/// The `Progress` trait, allowing to track which assets are
/// imported already.
pub trait Progress {
    /// The tracker this progress can create.
    type Tracker: Tracker;

    /// Add `num` assets to the progress.
    /// This should be done whenever a new asset is
    /// put in the queue.
    fn add_assets(&mut self, num: usize);

    /// Creates a `Tracker`.
    fn create_tracker(self) -> Self::Tracker;
}

impl Progress for () {
    type Tracker = ();

    fn add_assets(&mut self, _: usize) {}

    fn create_tracker(self) {}
}

/// A progress tracker which is passed to the `Loader`
/// in order to check how many assets are loaded.
#[derive(Default)]
pub struct ProgressCounter {
    errors: Arc<Mutex<Vec<AssetErrorMeta>>>,
    num_assets: usize,
    num_failed: Arc<AtomicUsize>,
    num_loading: Arc<AtomicUsize>,
}

impl ProgressCounter {
    /// Creates a new `Progress` struct.
    pub fn new() -> Self {
        Default::default()
    }

    /// Removes all errors and returns them.
    pub fn errors(&self) -> Vec<AssetErrorMeta> {
        let mut lock = self.errors.lock();
        let rv = lock.drain(..).collect();

        rv
    }

    /// Returns the number of assets this struct is tracking.
    pub fn num_assets(&self) -> usize {
        self.num_assets
    }

    /// Returns the number of assets that have failed.
    pub fn num_failed(&self) -> usize {
        self.num_failed.load(Ordering::Relaxed)
    }

    /// Returns the number of assets that are still loading.
    pub fn num_loading(&self) -> usize {
        self.num_loading.load(Ordering::Relaxed)
    }

    /// Returns the number of assets this struct is tracking.
    pub fn num_finished(&self) -> usize {
        self.num_assets - self.num_loading()
    }

    /// Returns `Completion::Complete` if all tracked assets are finished.
    pub fn complete(&self) -> Completion {
        match (
            self.num_failed.load(Ordering::Relaxed),
            self.num_loading.load(Ordering::Relaxed),
        ) {
            (0, 0) => Completion::Complete,
            (0, _) => Completion::Loading,
            (_, _) => Completion::Failed,
        }
    }

    /// Returns `true` if all assets have been imported without error.
    pub fn is_complete(&self) -> bool {
        self.complete() == Completion::Complete
    }
}

impl<'a> Progress for &'a mut ProgressCounter {
    type Tracker = ProgressCounterTracker;

    fn add_assets(&mut self, num: usize) {
        self.num_assets += num;
    }

    fn create_tracker(self) -> Self::Tracker {
        let errors = self.errors.clone();
        let num_failed = self.num_failed.clone();
        let num_loading = self.num_loading.clone();
        num_loading.fetch_add(1, Ordering::Relaxed);

        ProgressCounterTracker {
            errors,
            num_failed,
            num_loading,
        }
    }
}

/// Progress tracker for `ProgressCounter`.
#[derive(Default)]
pub struct ProgressCounterTracker {
    errors: Arc<Mutex<Vec<AssetErrorMeta>>>,
    num_failed: Arc<AtomicUsize>,
    num_loading: Arc<AtomicUsize>,
}

impl Tracker for ProgressCounterTracker {
    fn success(self: Box<Self>) {
        self.num_loading.fetch_sub(1, Ordering::Relaxed);
    }

    fn fail(
        self: Box<Self>,
        handle_id: u32,
        asset_type_name: &'static str,
        asset_name: String,
        error: Error,
    ) {
        show_error(handle_id, asset_type_name, &asset_name, &error);
        self.errors.lock().push(AssetErrorMeta {
            error,
            handle_id,
            asset_type_name,
            asset_name,
        });
        self.num_failed.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct AssetErrorMeta {
    pub error: Error,
    pub handle_id: u32,
    pub asset_type_name: &'static str,
    pub asset_name: String,
}

/// The `Tracker` trait which will be used by the loader to report
/// back to `Progress`.
pub trait Tracker: Send + 'static {
    // TODO: maybe add handles as parameters?
    /// Called if the asset could be imported.
    fn success(self: Box<Self>);
    /// Called if the asset couldn't be imported to an error.
    fn fail(
        self: Box<Self>,
        handle_id: u32,
        asset_type_name: &'static str,
        asset_name: String,
        error: Error,
    );
}

impl Tracker for () {
    fn success(self: Box<Self>) {}
    fn fail(
        self: Box<Self>,
        handle_id: u32,
        asset_type_name: &'static str,
        asset_name: String,
        error: Error,
    ) {
        show_error(handle_id, asset_type_name, &asset_name, &error);
        error!("Note: to handle the error, use a `Progress` other than `()`");
    }
}

fn show_error(handle_id: u32, asset_type_name: &'static str, asset_name: &String, error: &Error) {
    let mut err_out = format!(
        "Error loading handle {}, {}, with name {}, caused by: {:?}",
        handle_id, asset_type_name, asset_name, error
    );
    error
        .iter()
        .skip(1)
        .for_each(|e| err_out.push_str(&format!("\r\ncaused by: {:?}", e)));
    error!("{}", err_out);
}
