//! Types for constructing render passes.

#![allow(missing_docs)]
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;

use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use rayon_core;
use specs::SystemData;

use error::Result;
use pipe::{Effect, NewEffect, Target};
use types::{Encoder, Factory};

/// Is used to pass different `Encoder` and `Effect` into closure in different threads
pub struct Supplier<'a> {
    encoders: *mut [Encoder],
    effects: *mut [Effect],
    pd: PhantomData<&'a ()>,
}

impl<'a> Supplier<'a> {
    /// Create `Supplier` by passing enough `Encoder`s and `Effect`s
    /// The number is equal to thread count in `ThreadPool`
    fn new(encoders: &'a mut [Encoder], effects: &'a mut [Effect]) -> Self {
        Supplier {
            encoders: encoders,
            effects: effects,
            pd: PhantomData,
        }
    }

    fn index(&self) -> usize {
        rayon_core::current_thread_index().expect("Should be called from ThreadPool")
    }

    /// Dispense mutable references to `Encoder` and `Effect` for slice
    /// Different threads gets different pair
    /// unsafe due to ability to call mulitple times
    /// causing it to return multiple mutable references to the same `Encoder` and `Effect`
    /// `Apply` use this function once in each thread and 
    /// drops references before calling it again
    unsafe fn get(&self) -> (&mut Encoder, &mut Effect) {
        let slice = &mut *self.encoders;
        let count = slice.len();
        let encoder = slice
            .get_mut(self.index())
            .expect(&format!("Not enough objects. Index: {}, Supplier count: {}", self.index(), count));
        let slice = &mut *self.effects;
        let count = slice.len();
        let effect = slice
            .get_mut(self.index())
            .expect(&format!("Not enough objects. Index: {}, Supplier count: {}", self.index(), count));
        (encoder, effect)
    }
}

pub struct Apply<'a, I> {
    inner: I,
    supplier: Supplier<'a>,
}

impl<'a, I> ParallelIterator for Apply<'a, I>
    where I: ParallelIterator,
          I::Item: FnOnce(&mut Encoder, &mut Effect),
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<()>
    {
        let Apply { inner, supplier } = self;
        inner.map(move |f| {
            let (encoder, effect) = unsafe { supplier.get() };
            effect.clear();
            f(encoder, effect);
        }).drive_unindexed(consumer)
    }
}

impl<'a> Supplier<'a> {
    pub fn supply<I, F>(self, iter: I) -> Apply<'a, I>
        where I: ParallelIterator<Item=F>,
              F: FnOnce(&mut Encoder, &mut Effect) + Send,
    {
        Apply {
            inner: iter,
            supplier: self,
        }
    }
}

unsafe impl<'a> Send for Supplier<'a> {}
unsafe impl<'a> Sync for Supplier<'a> {}


pub trait PassApply<'a> {
    type Apply: ParallelIterator<Item=()>;
}

pub trait PassData<'a> {
    type Data: SystemData<'a> + Send;
}

pub trait Pass: for<'a> PassApply<'a> + for<'a> PassData<'a> + Send + Sync {
    fn compile(&self, effect: NewEffect) -> Result<Effect>;
    fn apply<'a, 'b: 'a>(&'a mut self, supplier: Supplier<'a>, data: <Self as PassData<'b>>::Data) -> <Self as PassApply<'a>>::Apply;
}

#[derive(Clone)]
pub struct CompiledPass<P> {
    effects: Vec<Effect>,
    inner: P,
}

impl<P> CompiledPass<P>
    where P: Pass,
{
    pub(super) fn compile(pass: P, fac: &mut Factory, out: &Target) -> Result<Self> {
        let effect = pass.compile(NewEffect::new(fac, out))?;
        Ok(CompiledPass {
            effects: vec![effect],
            inner: pass,
        })
    }
}

impl<P> CompiledPass<P> {
    pub fn apply<'a, 'b: 'a>(&'a mut self, encoders: &'a mut [Encoder], data: <P as PassData<'b>>::Data) -> <P as PassApply<'a>>::Apply
        where P: Pass,
    {
        if encoders.len() > self.effects.len() {
            let effect = self.effects[0].clone();
            self.effects.resize(encoders.len(), effect);
        }
        self.inner.apply(Supplier::new(encoders, &mut self.effects[..]), data)
    }
}

impl<P> Debug for CompiledPass<P> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("CompiledPass")
            .field("effects", &self.effects)
            .field("inner", &"[impl]")
            .finish()
    }
}
