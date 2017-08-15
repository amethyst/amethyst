//! Types for constructing render passes.

#![allow(missing_docs)]

use error::Result;
use pipe::{Effect, NewEffect, Target};
use scene::{Model, Scene};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use types::{Encoder, Factory};

pub trait Pass: Send + Sync {
    fn compile(&self, effect: NewEffect) -> Result<Effect>;
    fn apply(&self, enc: &mut Encoder, effect: &mut Effect, scene: &Scene, model: &Model);
}

#[derive(Clone)]
pub(crate) struct Description(Arc<Pass>);

impl Description {
    pub fn new<P: Pass + 'static>(pass: P) -> Self {
        Description(Arc::new(pass))
    }

    pub fn compile(self, fac: &mut Factory, out: &Target) -> Result<CompiledPass> {
        let eb = NewEffect::new(fac, out);
        let effect = self.0.compile(eb)?;
        Ok(CompiledPass {
            effect,
            inner: self.0,
        })
    }
}

impl Debug for Description {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple("PassDesc")
            .field(&"[impl]")
            .finish()
    }
}

#[derive(Clone)]
pub struct CompiledPass {
    effect: Effect,
    inner: Arc<Pass>,
}

impl CompiledPass {
    pub fn apply(&self, enc: &mut Encoder, scene: &Scene, model: &Model) {
        // TODO: Eliminate this clone.
        self.inner.apply(enc, &mut self.effect.clone(), scene, model);
    }
}

impl Debug for CompiledPass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("CompiledPass")
            .field("effect", &self.effect)
            .field("inner", &"[impl]")
            .finish()
    }
}
