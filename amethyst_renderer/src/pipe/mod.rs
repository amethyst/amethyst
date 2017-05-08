//! Renderer pipeline configuration.
//!
//! # Example
//!
//! ```ignore
//! let pipe = renderer.create_pipe(Pipeline::new()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_stage(Stage::with_target("gbuffer")
//!         .with_pass(ClearTarget::with_values([0.0; 1], 0.0))
//!         .with_pass(DrawFlat::new()))
//!     .with_stage(Stage::with_backbuffer()
//!         .with_pass(BlitBuffer::color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::compute_from("gbuffer"))))
//!     .expect("Could not build pipeline");
//! ```

pub use self::effect::{Effect, EffectBuilder};
pub use self::stage::{Stage, StageBuilder};
pub use self::target::{ColorBuffer, DepthBuffer, Target, TargetBuilder, Targets};

use error::Result;
use fnv::FnvHashMap as HashMap;
use std::sync::Arc;
use types::Factory;

pub mod pass;

mod effect;
mod stage;
mod target;

/// Defines how the rendering pipeline should be configured.
#[derive(Clone, Debug)]
pub struct Pipeline {
    stages: Vec<Stage>,
    targets: HashMap<String, Arc<Target>>,
}

impl Pipeline {
    /// Builds a new renderer pipeline.
    pub fn new() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Builds a default deferred pipeline.
    ///
    /// FIXME: Only generates a dummy pipeline for now.
    pub fn deferred() -> PipelineBuilder {
        use pass::*;
        PipelineBuilder::new()
            .with_target(Target::new("gbuffer")
                .with_num_color_bufs(4)
                .with_depth_buf(true))
            .with_stage(Stage::with_target("gbuffer")
                .with_pass(ClearTarget::with_values([1.0; 4], None)))
            .with_stage(Stage::with_backbuffer()
                .with_pass(ClearTarget::with_values([1.0; 4], None)))
    }

    /// Builds a default forward pipeline.
    ///
    /// FIXME: Only generates a dummy pipeline for now.
    pub fn forward() -> PipelineBuilder {
        use pass::*;
        PipelineBuilder::new()
            .with_stage(Stage::with_backbuffer()
                .with_pass(ClearTarget::with_values([1.0; 4], None)))
    }

    /// Returns an immutable slice of all stages in the pipeline.
    pub fn stages(&self) -> &[Stage] {
        self.stages.as_ref()
    }

    /// Returns an immutable reference to all targets and their name strings.
    pub fn targets(&self) -> &HashMap<String, Arc<Target>> {
        &self.targets
    }
}

/// Constructs a new pipeline with the given render targets and layers.
#[derive(Clone, Debug)]
pub struct PipelineBuilder {
    stages: Vec<StageBuilder>,
    targets: Vec<TargetBuilder>,
}

impl PipelineBuilder {
    /// Creates a new PipelineBuilder.
    pub fn new() -> PipelineBuilder {
        PipelineBuilder {
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
    #[doc(hidden)]
    pub fn build(self, fac: &mut Factory, out: &Arc<Target>) -> Result<Pipeline> {
        let mut targets = self.targets
            .iter()
            .cloned()
            .map(|tb| tb.build(fac, out.size()))
            .collect::<Result<Targets>>()?;

        targets.insert("".into(), out.clone());

        let stages = self.stages
            .iter()
            .cloned()
            .map(|sb| sb.build(fac, &targets))
            .collect::<Result<_>>()?;

        Ok(Pipeline {
           stages: stages,
           targets: targets,
        })
    }
}
