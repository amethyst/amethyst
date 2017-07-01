//! Types for constructing render passes.

#![allow(missing_docs)]

use error::Result;
use pipe::{Effect, EffectBuilder, Target, Targets};
use scene::Scene;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use types::{Encoder, Factory};

pub type FunctionFn = Arc<Fn(&mut Encoder, &Target) + Send + Sync>;
pub type PostFn = Arc<Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync>;

/// Discrete rendering pass.
#[derive(Clone)]
pub enum Pass {
    Function(FunctionFn),
    Post(PostFn, Effect),
}

impl Pass {
    /// Applies the rendering pass using the given `Encoder` and `Target`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target, scene: &Scene) {
        match *self {
            Pass::Function(ref func) => func(enc, out),
            Pass::Post(ref func, ref e) => func(enc, out, e, scene),
        }
    }
}

impl Debug for Pass {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Pass::Function(_) => {
                fmt.debug_tuple("Function")
                    .field(&"[closure]")
                    .finish()
            }
            Pass::Post(_, ref e) => {
                fmt.debug_tuple("Post")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}

#[derive(Clone)]
pub enum PassBuilder {
    Function(FunctionFn),
    Post(PostFn, EffectBuilder),
}

impl PassBuilder {
    pub fn function<F>(func: F) -> PassBuilder
        where F: Fn(&mut Encoder, &Target) + Send + Sync + 'static
    {
        PassBuilder::Function(Arc::new(func))
    }

    pub fn postproc<F>(eb: EffectBuilder, func: F) -> PassBuilder
        where F: Fn(&mut Encoder, &Target, &Effect, &Scene) + Send + Sync + 'static
    {
        PassBuilder::Post(Arc::new(func), eb)
    }

    pub(crate) fn finish(self, fac: &mut Factory, t: &Targets, out: &Target) -> Result<Pass> {
        match self {
            PassBuilder::Function(f) => Ok(Pass::Function(f)),
            PassBuilder::Post(f, e) => Ok(Pass::Post(f, e.finish(fac, out)?)),
        }
    }
}

impl Debug for PassBuilder {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            PassBuilder::Function(_) => {
                fmt.debug_tuple("Function")
                    .field(&"[closure]")
                    .finish()
            }
            PassBuilder::Post(_, ref e) => {
                fmt.debug_tuple("Post")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}
