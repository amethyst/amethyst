//! Renderer pipeline configuration.
//!
//! # Example
//!
//! ```rust,ignore
//! let pipe = renderer.create_pipe(Pipeline::build()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_stage(Stage::with_target("gbuffer")
//!         .clear_target([0.0; 1], 0.0)
//!         .draw_pass(DrawFlat::new()))
//!     .with_stage(Stage::with_backbuffer()
//!         .with_pass(BlitBuffer::color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::compute_from("gbuffer"))))
//!     .expect("Could not build pipeline");
//! ```

pub use self::effect::{DepthMode, Effect, EffectBuilder, NewEffect};
pub use self::stage::{Stage, StageBuilder};
pub use self::target::{ColorBuffer, DepthBuffer, Target, TargetBuilder, Targets};

use color::Rgba;
use error::Result;
use fnv::FnvHashMap as HashMap;
use std::iter::Filter;
use std::slice::Iter;
use types::Factory;

pub mod pass;

mod effect;
mod stage;
mod target;

/// Immutable iterator of pipeline stages.
#[derive(Debug)]
pub struct Stages<'s>(Filter<Iter<'s, Stage>, fn(&&Stage) -> bool>);

impl<'s> Iterator for Stages<'s> {
    type Item = &'s Stage;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Defines how the rendering pipeline should be configured.
#[derive(Clone, Debug)]
pub struct Pipeline {
    stages: Vec<Stage>,
    targets: HashMap<String, Target>,
}

impl Pipeline {
    /// Builds a new renderer pipeline.
    pub fn build() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Builds a default deferred pipeline.
    ///
    /// FIXME: Only generates a dummy pipeline for now.
    pub fn deferred() -> PipelineBuilder {
        PipelineBuilder::new()
            .with_target(Target::named("gbuffer")
                .with_num_color_bufs(4)
                .with_depth_buf(true))
            .with_stage(Stage::with_target("gbuffer")
                .clear_target(Rgba::black(), None))
            .with_stage(Stage::with_backbuffer()
                .clear_target(Rgba::black(), None))
    }

    /// Builds a default forward pipeline.
    ///
    /// FIXME: Only generates a dummy pipeline for now.
    pub fn forward<V>() -> PipelineBuilder
        where V: 'static + ::vertex::VertexFormat + ::vertex::WithField<::vertex::Position> + ::vertex::WithField<::vertex::TextureCoord>
    {
        PipelineBuilder::new()
            .with_stage(Stage::with_backbuffer()
                .clear_target([1.0; 4], None)
                .with_model_pass(::pass::DrawFlat::<V>::new()))
    }

    /// Iterates over all enabled stages in the pipeline.
    pub fn enabled_stages(&self) -> Stages {
        Stages(self.stages.iter().filter(|s| s.is_enabled()))
    }

    /// Returns an immutable reference to all targets and their name strings.
    pub fn targets(&self) -> &HashMap<String, Target> {
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
    pub fn new() -> Self {
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
    pub(crate) fn build(mut self, fac: &mut Factory, out: &Target) -> Result<Pipeline> {
        let mut targets = self.targets
            .drain(..)
            .map(|tb| tb.build(fac, out.size()))
            .collect::<Result<Targets>>()?;

        targets.insert("".into(), out.clone());

        let stages = self.stages
            .drain(..)
            .map(|sb| sb.build(fac, &targets))
            .collect::<Result<_>>()?;

        Ok(Pipeline {
           stages: stages,
           targets: targets,
        })
    }
}
