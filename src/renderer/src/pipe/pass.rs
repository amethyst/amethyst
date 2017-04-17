//! Different types of rendering passes.

#![allow(missing_docs)]

use error::Result;
use fnv::FnvHashMap as HashMap;
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use pipe::{Target, Targets};
use pipe::effect::{Effect, EffectBuilder};
use types::{Encoder, Factory, RawPipelineState, Sampler};
use scene::Scene;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

pub type FunctionFn = Arc<Fn(&mut Encoder, &Target) + Send + Sync>;
pub type PostprocFn = Arc<Fn(&mut Encoder, &Target, &Effect) + Send + Sync>;

/// Discrete rendering pass.
#[derive(Clone)]
pub enum Pass {
    Function(FunctionFn),
    Postproc(PostprocFn, Effect),
}

impl Pass {
    /// Applies the rendering pass using the given `Encoder` and `Target`.
    pub fn apply(&self, enc: &mut Encoder, out: &Target) {
        match *self {
            Pass::Function(ref f) => f(enc, out),
            Pass::Postproc(ref f, ref e) => f(enc, out, e),
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
            },
            Pass::Postproc(_, ref e) => {
                fmt.debug_tuple("Postproc")
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
    Postproc(PostprocFn, EffectBuilder),
}

impl PassBuilder {
    pub fn function<F>(func: F) -> PassBuilder
        where F: Fn(&mut Encoder, &Target) + Send + Sync + 'static
    {
        PassBuilder::Function(Arc::new(func))
    }

    pub fn postproc<F>(eb: EffectBuilder, func: F) -> PassBuilder
        where F: Fn(&mut Encoder, &Target, &Effect) + Send + Sync + 'static
    {
        PassBuilder::Postproc(Arc::new(func), eb)
    }

    pub fn build(self, fac: &mut Factory, t: &Targets, out: &Target) -> Result<Pass> {
        match self {
            PassBuilder::Function(f) => Ok(Pass::Function(f)),
            PassBuilder::Postproc(f, mut e) => Ok(Pass::Postproc(f, e.build(fac, out)?)),
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
            },
            PassBuilder::Postproc(_, ref e) => {
                fmt.debug_tuple("Postproc")
                    .field(&"[closure]")
                    .field(e)
                    .finish()
            }
        }
    }
}
