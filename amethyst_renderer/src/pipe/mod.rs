//! Renderer pipeline configuration.
//!
//! # Example
//!
//! ```rust,ignore
//! let pipe = renderer.create_pipe(PolyPipeline::build()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_stage(PolyStage::with_target("gbuffer")
//!         .clear_target([0.0; 1], 0.0)
//!         .draw_pass(DrawFlat::new()))
//!     .with_stage(PolyStage::with_backbuffer()
//!         .with_pass(BlitBuffer::color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::compute_from("gbuffer"))))
//!     .expect("Could not build pipeline");
//! ```

pub use self::{
    effect::{Data, DepthMode, Effect, EffectBuilder, Init, Meta, NewEffect},
    pipe::{Pipeline, PipelineBuild, PipelineBuilder, PipelineData, PolyPipeline, PolyStages},
    stage::{PolyStage, Stage, StageBuilder},
    target::{ColorBuffer, DepthBuffer, Target, TargetBuilder, Targets},
};

pub mod pass;

mod effect;
mod pipe;
mod stage;
mod target;
