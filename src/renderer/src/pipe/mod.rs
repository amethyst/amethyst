//! Renderer pipeline configuration.

pub use self::stage::{Stage, StageBuilder};
pub use self::target::{Target, TargetBuilder};

use {Factory, Result};
use fnv::FnvHashMap as HashMap;

mod stage;
mod target;

/// Defines how the rendering pipeline should be configured.
#[derive(Debug)]
pub struct Pipeline {
    stages: Vec<Stage>,
    targets: HashMap<String, Target>,
}

impl Pipeline {
    /// Returns an immutable slice of all stages in the pipeline.
    pub fn stages(&self) -> &[Stage] {
        self.stages.as_ref()
    }

    /// Returns an immutable reference to all targets and their name strings.
    pub fn targets(&self) -> &HashMap<String, Target> {
        &self.targets
    }
}

/// Constructs a new pipeline with the given render targets and layers.
pub struct PipelineBuilder<'a> {
    factory: &'a mut Factory,
    main_target: Target,
    stages: Vec<StageBuilder>,
    targets: Vec<TargetBuilder>,
}

impl<'a> PipelineBuilder<'a> {
    /// Creates a new PipelineBuilder with the given gfx::Factory.
    pub fn new(fac: &'a mut Factory, main: Target) -> Self {
        PipelineBuilder {
            factory: fac,
            main_target: main,
            stages: Vec::new(),
            targets: Vec::new(),
        }
    }

    /// Constructs a new render target for this pipeline.
    pub fn with_target(mut self, tb: TargetBuilder) -> Self {
        self.targets.push(tb);
        self
    }

    /// Constructs a new stage in this pipeline.
    pub fn with_stage(mut self, sb: StageBuilder) -> Self {
        self.stages.push(sb);
        self
    }

    /// Builds and returns the new pipeline.
    pub fn build(mut self) -> Result<Pipeline> {
        let factory = self.factory;
        let main_target = self.main_target;

        let mut targets = self.targets
            .drain(..)
            .map(|tb| tb.build(main_target.size(), factory))
            .collect::<Result<HashMap<String, Target>>>()?;

        targets.insert("".into(), main_target);

        let stages = self.stages
            .drain(..)
            .map(|sb| sb.build(&targets, factory))
            .collect::<Result<_>>()?;

        Ok(Pipeline {
            stages: stages,
            targets: targets,
        })
    }
}
