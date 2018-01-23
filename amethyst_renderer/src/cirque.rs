use std::collections::vec_deque::{Drain, VecDeque};
use std::ops::{Deref, DerefMut, Range};

use epoch::{CurrentEpoch, Epoch};

/// Circular queue to use with `Epoch` based resurces.
/// Useful when some resource may need to be changed while in use.
/// In this case new instance of the resource is allocated.
#[derive(Debug)]
pub struct Cirque<T> {
    values: VecDeque<(Epoch, T)>,
}

impl<T> Cirque<T> {
    /// Create new empty `Cirque`.
    pub fn new() -> Self {
        Cirque {
            values: VecDeque::new(),
        }
    }

    /// Create `Cirque` filled with items from `iter`.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let values = iter.into_iter().map(|value| (Epoch::new(), value));
        Cirque {
            values: values.collect(),
        }
    }

    /// Clear this queue.
    /// User must be sure that clearing items
    /// that are currently in use is safe.
    pub unsafe fn drain(&mut self) -> Drain<(Epoch, T)> {
        self.values.drain(..)
    }

    /// Get mutable entry for `span`.
    /// If first item in queue is not in use before `span.start` -
    /// makes it in use until `span.end`, puts in to the end of the queue
    /// and returns mutable reference to it.
    /// Otherwise returns vacant entry that can be used to add new instance.
    pub fn get_mut(&mut self, span: Range<Epoch>) -> EntryMut<T> {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            EntryMut::Vacant(VacantEntry {
                cirque: self,
                until: span.end,
            })
        } else {
            let (_, value) = self.values.pop_front().unwrap();
            self.values.push_back((span.end, value));
            let (_, ref mut value) = *self.values.back_mut().unwrap();
            EntryMut::Occupied(value)
        }
    }

    /// Get constant entry for `span`.
    /// If queue is not empty - marks the last item in queue to be in use until `span.end`
    /// and returns mutable reference to it.
    /// Otherwise returns vacant entry that can be used to add new instance.
    pub fn get(&mut self, span: Range<Epoch>) -> Entry<T> {
        if self.values.is_empty() {
            Entry::Vacant(VacantEntry {
                cirque: self,
                until: span.end,
            })
        } else {
            let (ref mut until, ref mut value) = *self.values.back_mut().unwrap();
            *until = span.end;
            Entry::Occupied(value)
        }
    }

    /// Returns the last item in queue as `Cirque::get` does
    /// or inserts new instance created by specified function.
    pub fn get_or_insert<F>(&mut self, span: Range<Epoch>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get(span) {
            Entry::Occupied(occupied) => occupied,
            Entry::Vacant(vacant) => vacant.insert(f()),
        }
    }
}

/// Entry that can be used to insert new instance into queue.
pub struct VacantEntry<'a, T: 'a> {
    cirque: &'a mut Cirque<T>,
    until: Epoch,
}

impl<'a, T: 'a> VacantEntry<'a, T> {
    /// Replace values in cirque with new ones.
    /// User must be sure that clearing items
    /// that are currently in use is safe.
    ///
    /// # Panics
    ///
    /// This function will panic if the iterator yields no items.
    pub unsafe fn replace<I>(self, iter: I) -> &'a mut T
    where
        I: IntoIterator<Item = T>,
    {
        self.cirque.values.clear();
        self.cirque
            .values
            .extend(iter.into_iter().map(|value| (Epoch::new(), value)));
        let (ref mut until, ref mut value) = *self.cirque
            .values
            .back_mut()
            .expect("Provided iterator must be non-empty");
        *until = self.until;
        value
    }

    /// Insert new value to the `Cirque`
    pub fn insert(self, value: T) -> &'a mut T {
        self.cirque.values.push_back((self.until, value));
        &mut self.cirque.values.back_mut().unwrap().1
    }
}

/// Mutable entry to the `Cirque`.
pub enum EntryMut<'a, T: 'a> {
    Vacant(VacantEntry<'a, T>),
    Occupied(&'a mut T),
}

impl<'a, T: 'a> EntryMut<'a, T> {
    /// Get `Some` reference from occupied entry.
    /// Returns `None` if it is `Vacant`.
    pub fn occupied(self) -> Option<&'a mut T> {
        match self {
            EntryMut::Occupied(occupied) => Some(occupied),
            EntryMut::Vacant(_) => None,
        }
    }
}

/// Immutable entry to the `Cirque`.
pub enum Entry<'a, T: 'a> {
    Vacant(VacantEntry<'a, T>),
    Occupied(&'a T),
}

impl<'a, T: 'a> Entry<'a, T> {
    /// Get `Some` reference from occupied entry.
    /// Returns `None` if it is `Vacant`.
    pub fn occupied(self) -> Option<&'a T> {
        match self {
            Entry::Occupied(occupied) => Some(occupied),
            Entry::Vacant(_) => None,
        }
    }
}
