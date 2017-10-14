use std::borrow::Borrow;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fnv::FnvHashMap;
use rayon::ThreadPool;

use {Asset, Directory, Format, Source};
use storage::{AssetStorage, Handle, Processed};

/// A unique store id, used to identify the storage in `AssetSpec`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(usize);

impl SourceId {
    /// Returns a copy of the internal id.
    pub fn id(&self) -> usize {
        self.0
    }
}

/// An `Allocator`, holding a counter for producing unique IDs.
#[derive(Debug, Default)]
pub struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    /// Produces a new id.
    pub fn next_id(&self) -> usize {
        self.store_count.fetch_add(1, Ordering::Relaxed)
    }
}

/// The asset loader, holding the contexts,
/// the default (directory) store and a reference to the
/// `ThreadPool`.
pub struct Loader {
    directory: Arc<Directory>,
    pool: Arc<ThreadPool>,
    sources: FnvHashMap<String, Arc<Source>>,
    //allocator: Allocator,
}

impl Loader {
    /// Creates a new asset loader, initializing the directory store with the
    /// given path.
    pub fn new<P>(directory: P, pool: Arc<ThreadPool>) -> Self
    where
        P: Into<PathBuf>,
    {
        Loader {
            directory: Arc::new(Directory::new(directory)),
            pool,
            sources: Default::default(),
        }
    }

    /// Add a source to the `Loader`, given an id and the source.
    pub fn add_source<I, S>(&mut self, id: I, source: S)
    where
        I: Into<String>,
        S: Source,
    {
        self.sources
            .insert(id.into(), Arc::new(source) as Arc<Source>);
    }

    /// Loads an asset with a given format from the default (directory) source.
    /// If you want to load from a custom source instead, use `load_from`.
    ///
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    pub fn load<A, F, N>(
        &self,
        id: N,
        format: F,
        progress: &mut Progress,
        storage: &AssetStorage<A>,
    ) -> Handle<A>
    where
        A: Asset,
        F: Format<A>,
        N: Into<String>,
    {
        self.load_from::<A, F, _, _>(id, format, "", progress, storage)
    }

    /// Loads an asset with a given id and format from a custom source.
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    pub fn load_from<A, F, N, S>(
        &self,
        name: N,
        format: F,
        source: &S,
        progress: &mut Progress,
        storage: &AssetStorage<A>,
    ) -> Handle<A>
    where
        A: Asset,
        F: Format<A> + 'static,
        N: Into<String>,
        S: AsRef<str> + Eq + Hash + ?Sized,
        String: Borrow<S>,
    {
        let source = match source.as_ref() {
            "" => self.directory.clone(),
            source => self.source(source),
        };

        progress.num_assets += 1;
        let progress_arc = progress.num_loading.clone();

        let handle = storage.allocate();
        let handle_clone = handle.clone();
        let processed = storage.processed.clone();

        let name = name.into();

        self.pool.spawn(move || {
            let data = format.import(name.clone(), source);
            processed.push(Processed {
                data,
                format: F::NAME.into(),
                handle,
                name,
            });
            drop(progress_arc);
        });

        handle_clone
    }

    /// Load an asset from data and return a handle.
    pub fn load_from_data<A>(&self, data: A::Data, storage: &AssetStorage<A>) -> Handle<A>
    where
        A: Asset,
    {
        let handle = storage.allocate();
        storage.processed.push(Processed {
            data: Ok(data),
            format: "".to_owned(),
            handle: handle.clone(),
            name: "<Data>".into(),
        });

        handle
    }

    fn source(&self, source: &str) -> Arc<Source> {
        self.sources
            .get(source)
            .expect("No such source. Maybe you forgot to add it with `Loader::add_source`?")
            .clone()
    }
}

/// A progress tracker which is passed to the `Loader`
/// in order to check how many asssets are loaded.
#[derive(Default)]
pub struct Progress {
    num_assets: usize,
    num_loading: Arc<()>,
}

impl Progress {
    /// Creates a new `Progress` struct.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the number of assets this struct is tracking.
    pub fn num_assets(&self) -> usize {
        self.num_assets
    }

    /// Returns the number of assets that are still loading.
    pub fn num_loading(&self) -> usize {
        Arc::strong_count(&self.num_loading) - 1
    }

    /// Returns the number of assets this struct is tracking.
    pub fn num_finished(&self) -> usize {
        self.num_assets - self.num_loading()
    }

    /// Returns `true` if all tracked assets are finished.
    pub fn is_complete(&self) -> bool {
        self.num_loading() == 0
    }
}
