//! A stage in the rendering pipeline.

use {Encoder, Error, Factory, Pass, Result, Scene, Target};
use fnv::FnvHashMap as HashMap;

/// A stage in the rendering pipeline.
#[derive(Debug)]
pub struct Stage {
    enabled: bool,
    passes: Vec<Box<Pass>>,
    target: Target,
}

impl Stage {
    /// Creates a new stage using the Target with the given name.
    pub fn with_target<T: Into<String>>(target_name: T) -> StageBuilder {
        StageBuilder::new(target_name)
    }

    /// Creates a new layer which draws straight into the backbuffer.
    pub fn with_main_target() -> StageBuilder {
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
    pub fn apply(&self, enc: &mut Encoder, scene: &Scene, delta: f64) {
        if self.enabled {
            for pass in self.passes.as_slice() {
                pass.apply(enc, &self.target, scene, delta);
            }
        }
    }
}

/// Constructs a new rendering stage.
pub struct StageBuilder {
    enabled: bool,
    passes: Vec<Box<Pass>>,
    target_name: String,
}

impl StageBuilder {
    /// Creates a new StageBuilder using the given target name.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            enabled: true,
            passes: Vec::new(),
            target_name: target_name.into(),
        }
    }

    /// Appends another pass to the stage.
    pub fn with_pass<P: Pass + 'static>(mut self, pass: P) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    /// Sets whether the stage is turned on by default.
    pub fn enabled_by_default(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    /// Builds and returns the stage.
    pub fn build(mut self, targets: &HashMap<String, Target>, fac: &mut Factory) -> Result<Stage> {
        use pass::Args;

        let name = self.target_name;
        let target = targets.get(&name).ok_or(Error::NoSuchTarget(name))?;

        let args = Args(fac, targets);

        let passes = self.passes
            .drain(..)
            .map(|mut p| p.init(&args).and(Ok(p)))
            .collect::<Result<_>>()?;

        Ok(Stage {
            enabled: self.enabled,
            passes: passes,
            target: target.clone(),
        })
    }
}
