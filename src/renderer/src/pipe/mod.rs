//! Renderer pipeline configuration.
//!
//! # Example
//!
//! ```ignore
//! let pipe = renderer.create_pipeline()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_stage(Stage::with_target("gbuffer")
//!         .with_pass(ClearTarget::with_values([0.0; 1], 0.0))
//!         .with_pass(DrawFlat::new()))
//!     .with_stage(Stage::with_backbuffer()
//!         .with_pass(BlitBuffer::color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::compute_from("gbuffer")))
//!     .build()
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
#[derive(Debug)]
pub struct Pipeline {
    stages: Vec<Stage>,
    targets: HashMap<String, Arc<Target>>,
}

impl Pipeline {
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
pub struct PipelineBuilder<'a> {
    factory: &'a mut Factory,
    main_target: Arc<Target>,
    stages: Vec<StageBuilder>,
    targets: Vec<TargetBuilder>,
}

impl<'a> PipelineBuilder<'a> {
    /// Creates a new PipelineBuilder with the given gfx::Factory.
    pub fn new(fac: &'a mut Factory, main: Arc<Target>) -> Self {
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
            .collect::<Result<Targets>>()?;

        targets.insert("".into(), main_target);

        let stages = self.stages
            .drain(..)
            .map(|sb| sb.build(factory, &targets))
            .collect::<Result<_>>()?;

        Ok(Pipeline {
            stages: stages,
            targets: targets,
        })
    }
}

/// Builds a default deferred pipeline.
///
/// FIXME: Only generates a dummy pipeline for now.
pub fn deferred(r: &mut super::Renderer) -> Result<Pipeline> {
    use pass::*;
    r.create_pipeline()
        .with_target(Target::new("gbuffer")
            .with_num_color_bufs(4)
            .with_depth_buf(true))
        .with_stage(Stage::with_target("gbuffer")
            .with_pass(ClearTarget::with_values([1.0; 4], None)))
        .with_stage(Stage::with_backbuffer()
            .with_pass(ClearTarget::with_values([1.0; 4], None)))
        .build()
}

/// Builds a default forward pipeline.
///
/// FIXME: Only generates a dummy pipeline for now.
pub fn forward(r: &mut super::Renderer) -> Result<Pipeline> {
    use pass::*;
    r.create_pipeline()
        .with_stage(Stage::with_backbuffer()
            .with_pass(ClearTarget::with_values([1.0; 4], None)))
        .build()
}
