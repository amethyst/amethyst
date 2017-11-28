

use std::cmp::{Ordering, PartialOrd, max, min};
use std::collections::VecDeque;
use std::ops::{Add, Deref, DerefMut};

use relevant::Relevant;

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


pub trait ValidThrough {
    /// Encapsulated data.
    type Data;

    /// Get last epoch this value has to be valid through.
    fn valid_through(&self) -> Epoch;

    /// Try to dispose of this value.
    fn dispose(self, current: &CurrentEpoch) -> Result<Self::Data, Self> where Self: Sized;
}

/// Check if this value valid through specified `Epoch`
fn is_valid_through<T: ValidThrough>(value: &T, epoch: Epoch) -> bool {
    value.valid_through() <= epoch
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
/// is equal to last `Epoch` spcified in `make_valid_through` and `borrow`
pub struct Eh<T> {
    relevant: Relevant,
    ptr: *const T,
    valid_through: u64,
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
    pub fn make_valid_through(this: &mut Self, epoch: Epoch) {
        this.valid_through = max(this.valid_through, epoch.0);
    }

    /// Borrow `Ec` from this `Eh`
    /// `Ec` will expire after specified `Epoch`
    pub fn borrow(this: &mut Self, epoch: Epoch) -> Ec<T> {
        Self::make_valid_through(this, epoch);
        Ec {
            ptr: this.ptr,
            valid_through: this.valid_through,
        }
    }
}

impl<T> ValidThrough for Eh<T> {
    type Data = T;

    fn valid_through(&self) -> Epoch {
        Epoch(self.valid_through)
    }

    fn dispose(self, current: &CurrentEpoch) -> Result<T, Self> {
        if self.valid_through < current.0 {
            self.relevant.dispose();
            Ok(unsafe { *Box::from_raw(self.ptr as *mut _) })
        } else {
            Err(self)
        }
    }
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

impl<T> Deref for Eh<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

pub struct DeletionQueue<T> {
    offset: u64,
    queue: VecDeque<Vec<T>>,
    clean_vecs: Vec<Vec<T>>,
}

impl<T> DeletionQueue<T>
where
    T: ValidThrough
{
    pub fn new() -> Self {
        DeletionQueue {
            offset: 0,
            queue: VecDeque::new(),
            clean_vecs: Vec::new(),
        }
    }

    pub fn add(&mut self, value: T) {
        let index = (value.valid_through().0 - self.offset) as usize;
        let ref mut queue = self.queue;
        let ref mut clean_vecs = self.clean_vecs;

        let len = queue.len();
        queue.extend((len..index).map(|_| {
            clean_vecs.pop().unwrap_or_else(|| Vec::new())
        }));
        queue[index].push(value);
    }

    pub fn clean<F>(&mut self, current: &CurrentEpoch, mut f: F)
    where
        F: FnMut(T::Data),
    {
        let index = (current.now().0 - self.offset) as usize;
        let len = self.queue.len();

        for mut vec in self.queue.drain(..min(index, len)) {
            for value in vec.drain(..) {
                assert!(!is_valid_through(&value, current.now()));
                f(value.dispose(current).unwrap_or_else(|_| unreachable!()));
            }
            self.clean_vecs.push(vec);
        }
        self.offset += index as u64;
    }
}
