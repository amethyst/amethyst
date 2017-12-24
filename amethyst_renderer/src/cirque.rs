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

    pub fn create<I>(iter: I) -> Self
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

    pub fn get(&mut self, span: Range<Epoch>) -> Option<CirqueRef<T>> {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            return None;
        }

        let value = self.values.pop_front().unwrap().1;
        Some(CirqueRef {
            cirque: self,
            value: Some(value),
            until: span.end,
        })
    }

    pub fn get_or_try_insert<F, E>(&mut self, span: Range<Epoch>, mut f: F) -> Result<CirqueRef<T>, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            let count = span.end - span.start;
            let len = self.values.len() as u64;
            for _ in len .. count {
                self.values.push_front((Epoch::new(), f()?));
            }
        }

        let value = self.values.pop_front().unwrap().1;
        Ok(CirqueRef {
            cirque: self,
            value: Some(value),
            until: span.end,
        })
    }

    pub fn get_or_insert<F>(&mut self, span: Range<Epoch>, mut f: F) -> CirqueRef<T>
    where
        F: FnMut() -> T,
    {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            let count = span.end - span.start;
            let len = self.values.len() as u64;
            for _ in len .. count {
                self.values.push_front((Epoch::new(), f()));
            }
        }

        let value = self.values.pop_front().unwrap().1;
        CirqueRef {
            cirque: self,
            value: Some(value),
            until: span.end,
        }
    }

    pub fn get_or_try_replace<F, I, E>(&mut self, span: Range<Epoch>, f: F) -> Result<CirqueRef<T>, E>
    where
        F: FnOnce(usize) -> Result<I, E>,
        I: IntoIterator<Item=T>,
    {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            let count = span.end - span.start;
            let mut new = VecDeque::new();
            new.extend(f(count as usize)?.into_iter().map(|v| (Epoch::new(), v)));
            self.values = new;
        }

        let value = self.values.pop_front().unwrap().1;
        Ok(CirqueRef {
            cirque: self,
            value: Some(value),
            until: span.end,
        })
    }

    pub fn get_or_replace<F, I>(&mut self, span: Range<Epoch>, f: F) -> CirqueRef<T>
    where
        F: FnOnce(usize) -> I,
        I: IntoIterator<Item=T>,
    {
        if self.values
            .front()
            .map(|&(until, _)| until >= span.start)
            .unwrap_or(true)
        {
            let count = span.end - span.start;
            self.values.clear();
            self.values.extend(f(count as usize).into_iter().map(|v| (Epoch::new(), v)));
        }

        let value = self.values.pop_front().unwrap().1;
        CirqueRef {
            cirque: self,
            value: Some(value),
            until: span.end,
        }
    }

    pub fn last(&self) -> Option<&T> {
        self.values.back().map(|&(_, ref value)| value)
    }

    pub unsafe fn take(&mut self) -> VecDeque<(Epoch, T)> {
        ::std::mem::replace(&mut self.values, VecDeque::new())
    }
}

pub struct CirqueRef<'a, T: 'a> {
    cirque: &'a mut Cirque<T>,
    value: Option<T>,
    until: Epoch,
}

impl<'a, T> Deref for CirqueRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for CirqueRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}

impl<'a, T> Drop for CirqueRef<'a, T> {
    fn drop(&mut self) {
        self.cirque.values.push_back((self.until, self.value.take().unwrap()));
    }
}
