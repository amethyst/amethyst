

use std::cmp::{max, PartialOrd};
use std::ops::Deref;

use specs::{Fetch, FetchMut};

type EpochData<'a> = Fetch<'a, EpochCounter>;
type EpochDataMut<'a> = FetchMut<'a, EpochCounter>;

/// Epoch identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);

impl PartialOrd<EpochCounter> for Epoch {
    #[inline]
    fn partial_cmp(&self, other: &EpochCounter) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq<EpochCounter> for Epoch {
    #[inline]
    fn eq(&self, other: &EpochCounter) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq<EpochCounter> for Epoch {}

impl Ord<EpochCounter> for Epoch {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
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
pub struct EpochCounter(u64);

impl EpochCounter {
    fn now(&self) -> Epoch {
        Epoch(self.0)
    }
}

/// Weak epoch pointer to `T`.
/// It will expire after some `Epoch`.
struct Ec<T> {
    ptr: *const T,
    valid_for: Epoch,
}

impl<T> Ec<T> {
    /// Get `Epoch` after which this `Ec` will expire.
    fn valid_for(&self) -> Epoch {
        self.valid_for
    }

    /// Get reference to the pointer value.
    /// Returns `Some` if `Ec` hasn't expired yet (EpochCounter is less than `self.valid_for()`).
    /// Returns `None` otherwise
    #[inline]
    fn get<'a>(&'a self, ec: &EpochCounter) -> Option<&'a T> {
        if self.valid_for < ec {
            unsafe { Some(&*self.ptr) }
        } else {
            None
        }
    }
}

/// Strong epoch pointer to `T`.
/// It will hold value alive until dropped
/// and 
struct Eh<T> {
    ptr: *const T,
    drop_after: Epoch,
}

impl<T> From<Box<T>> for Eh<T> {
    fn from(b: Box<T>) -> Self {
        Eh {
            ptr: Box::into_raw(b),
            drop_after: Epoch(0)
        }
    }
}

impl<T> Eh<T> {
    /// Wrap value into `Eh`
    fn new(value: T) -> Self {
        Eh {
            ptr: Box::into_raw(Box::new(value)),
            drop_after: Epoch(0)
        }
    }

    /// Make all new `Ec` from this to be valid
    /// until specifyed `Epoch` expired
    fn make_valid_for(this: &mut Self, epoch: Epoch) {
        this.drop_after = max(this.drop_after, epoch);
    }

    /// Get last epoch for which `Eh` whould be valid
    fn valid_for(this: &Self) -> Epoch {
        this.drop_after
    }

    /// Borrow `Ec` from this `Eh`
    /// `Ec` will expire after specified `Epoch`
    fn borrow(this: &mut Self, epoch: Epoch) -> Ec<T> {
        this.make_valid_for(epoch);
        Ec {
            ptr: this.ptr,
            valid_for: this.drop_after,
        }
    }
}

impl<T> Deref for Eh<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}
