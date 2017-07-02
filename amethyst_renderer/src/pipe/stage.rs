//! A stage in the rendering pipeline.

use error::{Error, Result};
use pipe::{Target, Targets};
use pipe::pass::{Pass, PassBuilder};
use scene::{Model, Scene};
use std::sync::Arc;
use types::{Encoder, Factory};

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage {
    enabled: bool,
    passes: Vec<Pass>,
    target: Arc<Target>,
}

impl Stage {
    /// Creates a new stage using the Target with the given name.
    pub fn with_target<T: Into<String>>(target_name: T) -> StageBuilder {
        StageBuilder::new(target_name.into())
    }

    /// Creates a new layer which draws straight into the backbuffer.
    pub fn with_backbuffer() -> StageBuilder {
        StageBuilder::new("")
    }

    /// Sets whether this layer should execute.
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Checks whether this layer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Applies all passes in this stage to the given `Scene` and outputs the
    /// result to the proper target.
    pub fn apply(&self, enc: &mut Encoder, model: &Model, scene: &Scene) {
        if self.enabled {
            for pass in self.passes.as_slice() {
                pass.apply(enc, model, scene, &self.target);
            }
        }
    }
}

/// Constructs a new rendering stage.
#[derive(Clone, Debug)]
pub struct StageBuilder {
    enabled: bool,
    passes: Vec<PassBuilder>,
    target_name: String,
}

impl StageBuilder {
    /// Creates a new `StageBuilder` using the given target.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            enabled: true,
            passes: Vec::new(),
            target_name: target_name.into(),
        }
    }

    /// Appends another `Pass` to the stage.
    pub fn with_pass<P: Into<PassBuilder>>(mut self, pass: P) -> Self {
        self.passes.push(pass.into());
        self
    }

    /// Sets whether the `Stage` is turned on by default.
    pub fn enabled(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    /// Builds and returns the stage.
    #[doc(hidden)]
    pub(crate) fn finish(mut self, fac: &mut Factory, targets: &Targets) -> Result<Stage> {
        let name = self.target_name;
        let out = targets
            .get(&name)
            .cloned()
            .ok_or(Error::NoSuchTarget(name))?;

        let passes = self.passes
            .drain(..)
            .map(|pb| pb.finish(fac, targets, &out))
            .collect::<Result<_>>()?;

        Ok(Stage {
            enabled: self.enabled,
            passes: passes,
            target: out,
        })
    }
}
