use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use amethyst_error::Error;
use log::error;
use parking_lot::Mutex;

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
#[derive(Default, Debug)]
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
        lock.drain(..).collect()
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

    /// Returns the number of assets that have successfully loaded.
    pub fn num_finished(&self) -> usize {
        self.num_assets - self.num_loading() - self.num_failed()
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
#[derive(Default, Debug)]
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
        handle_id: u64,
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

        // Failed assets are not requeued for loading, so we subtract it from the number that tracks
        // the assets that are still loading.
        self.num_loading.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct AssetErrorMeta {
    pub error: Error,
    pub handle_id: u64,
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
        handle_id: u64,
        asset_type_name: &'static str,
        asset_name: String,
        error: Error,
    );
}

impl Tracker for () {
    fn success(self: Box<Self>) {}
    fn fail(
        self: Box<Self>,
        handle_id: u64,
        asset_type_name: &'static str,
        asset_name: String,
        error: Error,
    ) {
        show_error(handle_id, asset_type_name, &asset_name, &error);
        error!("Note: to handle the error, use a `Progress` other than `()`");
    }
}

fn show_error(handle_id: u64, asset_type_name: &'static str, asset_name: &str, error: &Error) {
    let mut err_out = format!(
        "Error loading handle {}, {}, with name {}: {}",
        handle_id, asset_type_name, asset_name, error,
    );
    error
        .causes()
        .for_each(|e| err_out.push_str(&format!("\ncaused by: {}\n{:?}", e, e)));
    error!("{}", err_out);
}

#[cfg(test)]
mod tests {
    use amethyst_error::Error;

    use super::{Completion, Progress, ProgressCounter, Tracker};

    #[test]
    fn progress_counter_complete_returns_correct_completion_status_when_loading_or_complete() {
        let mut progress_counter = ProgressCounter::new();
        let mut progress = &mut progress_counter;
        progress.add_assets(2);
        let tracker_0 = Box::new(progress.create_tracker());
        let tracker_1 = Box::new(progress.create_tracker());

        // 2 loading, 0 success
        assert_eq!(Completion::Loading, progress.complete());
        assert!(!progress.is_complete());

        // 1 loading, 1 success
        tracker_0.success();
        assert_eq!(Completion::Loading, progress.complete());
        assert!(!progress.is_complete());

        // 0 loading, 2 success
        tracker_1.success();
        assert_eq!(Completion::Complete, progress.complete());
        assert!(progress.is_complete());
    }

    #[test]
    fn progress_counter_complete_returns_failed_when_any_assets_failed() {
        let mut progress_counter = ProgressCounter::new();
        let mut progress = &mut progress_counter;
        progress.add_assets(2);
        let tracker_0 = Box::new(progress.create_tracker());
        let tracker_1 = Box::new(progress.create_tracker());

        // 1 failed, 1 loading
        tracker_0.fail(
            1,
            "AssetType",
            String::from("test.asset"),
            Error::from_string(""),
        );
        assert_eq!(Completion::Failed, progress.complete());
        assert!(!progress.is_complete());

        // 1 failed, 1 success
        tracker_1.success();
        assert_eq!(Completion::Failed, progress.complete());
        assert!(!progress.is_complete());
    }

    #[test]
    fn progress_counter_num_finished_excludes_loading_and_failed_assets() {
        let mut progress_counter = ProgressCounter::new();
        let mut progress = &mut progress_counter;
        progress.add_assets(3);
        let tracker_0 = Box::new(progress.create_tracker());
        let tracker_1 = Box::new(progress.create_tracker());
        let tracker_2 = Box::new(progress.create_tracker());

        // 0 failed, 3 loading, 0 success
        assert_eq!(0, progress.num_finished());

        // 1 failed, 1 loading, 0 success
        tracker_0.fail(
            1,
            "AssetType",
            String::from("test.asset"),
            Error::from_string(""),
        );
        assert_eq!(0, progress.num_finished());

        // 1 failed, 1 loading, 1 success
        tracker_1.success();
        assert_eq!(1, progress.num_finished());

        // 1 failed, 0 loading, 2 success
        tracker_2.success();
        assert_eq!(2, progress.num_finished());
    }
}
