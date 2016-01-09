//! Structures representing static GPU pipeline state.

use renderer::types::*;

pub struct BlenderInfo {
    pub alpha_to_coverage_enabled: bool,
    pub dual_source_enabled: bool,
    pub blend_operation: LogicOp,
    pub targets: Vec<TargetInfo>,
}

pub struct DepthStencilTesterInfo {
    pub format_channel: u32,
    pub format_numeric: u32,
}

pub struct InputAssemblerInfo {
    pub vertex_reuse_enabled: bool,
    pub primitive_used: Primitive,
}

pub struct RasterizerInfo {
    pub depth_clip_enabled: bool,
}

pub struct TargetInfo {
    pub blending_enabled: bool,
    pub channel_write_mask: u8,
    pub format_channel: u32,
    pub format_numeric: u32,
}

pub struct PipelineInfo {
    pub color_blender: BlenderInfo,
    pub depth_stencil: DepthStencilTesterInfo,
    pub input_assembler: InputAssemblerInfo,
    pub rasterizer: RasterizerInfo,
    pub shaders: ShaderSet,
}

/// Handle to a pipeline state object.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pipeline(u64);
