//! Defines `Relevant` type to use in types that requires
//! custom dealocation.
//!

use hibitset::BitSetLike;
use specs::{Index, UnprotectedStorage};

/// Values of this type can't be automatically dropped.
/// If struct or enum has field with type `Relevant`,
/// it can't be automatically dropped either. And so considered relevant too.
/// User has to deconstruct such values and call `Relevant::dispose`.
/// If relevant field is private it means that user has to move value into some public method.
/// For example `memory::Block` should be returned to the `MemoryAllocator` it came from.
///
/// User of the engine won't usually deal with real relevant types.
/// More often user will face wrappers that has backdoor - some technique
/// to dispose internal relevant fields with runtime cost.
/// In debug mode such wrappers can put warnings in log.
/// So that user will know they should be disposed manually.
///
/// # Panics
///
/// Panics when dropped.
///
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Relevant;

impl Relevant {
    /// Dispose this value.
    pub fn dispose(self) {
        ::std::mem::forget(self)
    }
}

impl Drop for Relevant {
    fn drop(&mut self) {
        println!("Values of this type can't be dropped!")
    }
}

pub struct RelevantStorage<S, T> {
    inner: S,
    dropped: Vec<T>,
}

impl<S, T> RelevantStorage<S, T> {
    pub fn new(storage: S) -> Self {
        RelevantStorage {
            inner: storage,
            dropped: Vec::new(),
        }
    }

    pub fn drain(&mut self) -> ::std::vec::Drain<T> {
        self.dropped.drain(..)
    }
}

impl<S, T> Default for RelevantStorage<S, T>
where
    S: Default,
{
    fn default() -> Self {
        RelevantStorage {
            inner: S::default(),
            dropped: Vec::new(),
        }
    }
}

impl<S, T> UnprotectedStorage<T> for RelevantStorage<S, T>
where
    S: UnprotectedStorage<T>,
{
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        for i in has.iter() {
            self.dropped.push(self.inner.remove(i as Index));
        }
    }
    unsafe fn get(&self, id: Index) -> &T {
        self.inner.get(id)
    }
    unsafe fn get_mut(&mut self, id: Index) -> &mut T {
        self.inner.get_mut(id)
    }
    unsafe fn insert(&mut self, id: Index, value: T) {
        self.inner.insert(id, value)
    }
    unsafe fn remove(&mut self, id: Index) -> T {
        self.inner.remove(id)
    }
    unsafe fn drop(&mut self, id: Index) {
        self.dropped.push(self.inner.remove(id));
    }
}
