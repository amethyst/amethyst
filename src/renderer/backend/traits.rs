//! Traits used by all renderer backends.

use renderer::ir::CommandBuffer;
use renderer::backend::state_dynamic::*;
use renderer::backend::state_static::{Pipeline, PipelineInfo};
use renderer::types::*;

/// Trait implemented by renderer backends.
pub trait Backend {
    fn process(&mut self, buffers: Vec<CommandBuffer>);
}

/// Trait for managing handles to GPU resources.
pub trait Resources {
    fn create_buffer(&mut self, info: BufferInfo) -> Option<Buffer>;
    fn create_shader(&mut self) -> Option<Shader>;
}

/// Trait for managing handles to GPU state objects.
pub trait States {
    fn create_blend(&mut self, info: BlendInfo) -> Option<DynamicState>;
    fn create_depth_stencil(&mut self, info: DepthStencilInfo)
                                                    -> Option<DynamicState>;
    fn create_pipeline(&mut self, info: PipelineInfo) -> Option<Pipeline>;
    fn create_raster(&mut self, info: RasterizerInfo) -> Option<DynamicState>;
    fn create_viewport(&mut self, info: ViewportInfo) -> Option<DynamicState>;
}

