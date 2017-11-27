

use std::cmp::{Ordering, PartialOrd, max};
use std::ops::{Add, Deref};

use specs::{Fetch, FetchMut};
use relevant::Relevant;

type EpochData<'a> = Fetch<'a, CurrentEpoch>;
type EpochDataMut<'a> = FetchMut<'a, CurrentEpoch>;

/// Epoch identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);

impl Epoch {
    pub fn new() -> Self {
        Epoch(0)
    }
}

impl Add<u64> for Epoch {
    type Output = Epoch;
    fn add(self, add: u64) -> Epoch {
        Epoch(self.0 + add)
    }
}

/// Epoch counter.
/// Place it somewhere where all `Ec` users can access it
#[derive(Debug)]
pub struct CurrentEpoch(u64);

impl CurrentEpoch {
    pub fn new() -> Self {
        CurrentEpoch(1)
    }

    pub fn now(&self) -> Epoch {
        Epoch(self.0)
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

/// Validation marker.
/// It say that marked item is valid through the `Epoch` but may become invalid after.
/// Primarily to use in `Ec` and `Eh` or any custom "Epoch countered" resource wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidThrough(pub u64);

impl ValidThrough {
    pub fn new() -> Self {
        ValidThrough(0)
    }

    /// Make marer `ValidThrough` next `amount` of `Epoch`s
    pub fn for_next(current: &CurrentEpoch, amount: u64) -> Self {
        ValidThrough(current.0 + amount)
    }

    /// Check if marked item is valid in current `Epoch`
    pub fn is_valid(&self, current: &CurrentEpoch) -> bool {
        self.0 <= current.0
    }
}

/// Weak epoch pointer to `T`.
/// It will expire after some `Epoch`.
pub struct Ec<T> {
    ptr: *const T,
    valid_through: u64,
}

impl<T> Ec<T> {
    /// Get `Epoch` after which this `Ec` will expire.
    pub fn valid_through(&self) -> Epoch {
        Epoch(self.valid_through)
    }

    /// Get reference to the pointer value.
    /// Returns `Some` if `Ec` hasn't expired yet
    /// (CurrentEpoch is less than `self.valid_through()`).
    /// Returns `None` otherwise.
    #[inline]
    pub fn get<'a>(&'a self, current: &CurrentEpoch) -> Option<&'a T> {
        if self.valid_through <= current.0 {
            unsafe { Some(&*self.ptr) }
        } else {
            None
        }
    }
}

/// Strong epoch pointer to `T`.
/// It will hold value alive and can't be dropped until `CurrentEpoch`
/// is equal to last `Epoch` spcified in `make_valid_for` and `borrow`
pub struct Eh<T> {
    relevant: Relevant,
    ptr: *const T,
    valid_through: u64,
}

impl<T> From<Box<T>> for Eh<T> {
    fn from(b: Box<T>) -> Self {
        Eh {
            relevant: Relevant,
            ptr: Box::into_raw(b),
            valid_through: 0,
        }
    }
}

impl<T> Eh<T> {
    /// Wrap value into `Eh`
    pub fn new(value: T) -> Self {
        Eh {
            relevant: Relevant,
            ptr: Box::into_raw(Box::new(value)),
            valid_through: 0,
        }
    }

    /// Make all new `Ec` from this to be valid
    /// until specifyed `Epoch` expired
    pub fn make_valid_for(this: &mut Self, epoch: Epoch) {
        this.valid_through = max(this.valid_through, epoch.0);
    }

    /// Get last epoch for which `Eh` whould be valid
    pub fn valid_through(this: &Self) -> Epoch {
        Epoch(this.valid_through)
    }

    /// Borrow `Ec` from this `Eh`
    /// `Ec` will expire after specified `Epoch`
    pub fn borrow(this: &mut Self, epoch: Epoch) -> Ec<T> {
        Self::make_valid_for(this, epoch);
        Ec {
            ptr: this.ptr,
            valid_through: this.valid_through,
        }
    }

    ///
    pub fn dispose(self, current: &CurrentEpoch) {
        assert!(self.valid_through < current.0);
        unsafe { self.relevant.dispose() }
    }
}

impl<T> Deref for Eh<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}
