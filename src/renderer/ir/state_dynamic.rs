//! Structures representing dynamic GPU pipeline state.

use renderer::types::*;

pub struct BlendInfo {
    pub blend_constants: [f32; 4],
    pub targets: Vec<TargetBlendInfo>,
}

pub struct DepthStencilInfo {
    pub depth_func: CompareFunc,
    pub depth_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_bounds_enabled: bool,
    pub max_depth: f32,
    pub min_depth: f32,
    pub stencil_enabled: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub back: DepthStencilOp,
    pub front: DepthStencilOp,
}

pub struct RasterizerInfo {
    pub cull_mode: CullMode,
    pub depth_bias: i32,
    pub depth_bias_slope_scaled: f32,
    pub depth_bias_clamp: f32,
    pub fill_mode: FillMode,
    pub winding_order: Winding,
}

pub struct TargetBlendInfo {
    pub blending_enabled: bool,
    pub alpha_func: BlendFunc,
    pub color_func: BlendFunc,
    pub dest_alpha: Blend,
    pub dest_color: Blend,
    pub source_alpha: Blend,
    pub source_color: Blend,
}

pub struct ViewportInfo {
    pub scissor_test_enabled: bool,
    pub scissors: Vec<ScissorBox>,
    pub viewports: Vec<Viewport>,
}

/// Handle to a dynamic state object.
#[derive(Clone, Eq, PartialEq)]
pub enum DynamicState {
    /// Color blend state.
    Blend(u64),
    /// Depth stencil state.
    DepthStencil(u64),
    /// Rasterizer state.
    Rasterizer(u64),
    /// Viewport state.
    Viewport(u64),
}
