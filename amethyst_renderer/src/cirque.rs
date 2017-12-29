use std::collections::VecDeque;
use std::ops::{Deref, DerefMut, Range};

use epoch::{Epoch};

#[derive(Debug)]
pub struct Cirque<T> {
    values: VecDeque<(Epoch, T)>,
}

impl<T> Cirque<T> {
    pub fn new() -> Self {
        Cirque {
            values: VecDeque::new(),
        }
    }

    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=T>,
    {
        let values = iter.into_iter().map(|value| (Epoch::new(), value));
        Cirque {
            values: values.collect(),
        }
    }

    pub fn clear(&mut self) {
        self.values.clear()
    }

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

    pub fn get_or_insert<F>(&mut self, span: Range<Epoch>, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get(span) {
            Entry::Occupied(occupied) => occupied,
            Entry::Vacant(vacant) => vacant.insert(f())
        }
    }

    pub unsafe fn take(&mut self) -> VecDeque<(Epoch, T)> {
        ::std::mem::replace(&mut self.values, VecDeque::new())
    }
}

pub struct VacantEntry<'a, T: 'a> {
    cirque: &'a mut Cirque<T>,
    until: Epoch,
}

impl<'a, T: 'a> VacantEntry<'a, T> {
    /// Replace values in cirque with new ones.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the iterator yields no items.
    pub fn replace<I>(self, iter: I) -> &'a mut T
    where
        I: IntoIterator<Item=T>,
    {
        self.cirque.values.clear();
        self.cirque.values.extend(iter.into_iter().map(|value| (Epoch::new(), value)));
        let (ref mut until, ref mut value) = *self.cirque.values.back_mut().expect("Provided iterator must be non-empty");
        *until = self.until;
        value
    }

    /// Insert new value to the `Cirque`
    pub fn insert(self, value: T) -> &'a mut T {
        self.cirque.values.push_back((self.until, value));
        &mut self.cirque.values.back_mut().unwrap().1
    }
}

pub enum EntryMut<'a, T: 'a> {
    Vacant(VacantEntry<'a, T>),
    Occupied(&'a mut T),
}

impl<'a, T: 'a> EntryMut<'a, T> {
    pub fn occupied(self) -> Option<&'a mut T> {
        match self {
            EntryMut::Occupied(occupied) => Some(occupied),
            EntryMut::Vacant(_) => None,
        }
    }
}

pub enum Entry<'a, T: 'a> {
    Vacant(VacantEntry<'a, T>),
    Occupied(&'a T),
}

impl<'a, T: 'a> Entry<'a, T> {
    pub fn occupied(self) -> Option<&'a T> {
        match self {
            Entry::Occupied(occupied) => Some(occupied),
            Entry::Vacant(_) => None,
        }
    }
}
