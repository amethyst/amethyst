//! A data parallel rendering engine developed by the [Amethyst][am] project.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/develop/src/renderer
//! [bk]: https://www.amethyst.rs/book/master/

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate genmesh;
extern crate gfx;
extern crate gfx_core;
#[macro_use]
extern crate gfx_macros;
#[macro_use]
extern crate glsl_layout;
extern crate hetseq;
extern crate hibitset;
extern crate imagefmt;
#[macro_use]
extern crate log;
extern crate rayon;
#[macro_use]
extern crate serde;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate smallvec;
extern crate wavefront_obj;
extern crate winit;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_device_dx11;
#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_window_dxgi;

#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_device_metal;
#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_window_metal;

#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

#[cfg(feature = "vulkan")]
extern crate gfx_device_vulkan;
#[cfg(feature = "vulkan")]
extern crate gfx_window_vulkan;

pub use bundle::RenderBundle;
pub use cam::{ActiveCamera, ActiveCameraPrefab, Camera, CameraPrefab, Projection};
pub use color::Rgba;
pub use config::DisplayConfig;
pub use formats::{
    build_mesh_with_combo, create_mesh_asset, create_texture_asset, BmpFormat, ComboMeshCreator,
    GraphicsPrefab, ImageData, ImageError, JpgFormat, MaterialPrefab, MeshCreator, MeshData,
    ObjFormat, PngFormat, TextureData, TextureFormat, TextureMetadata, TexturePrefab,
};
pub use input::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use light::{DirectionalLight, Light, LightPrefab, PointLight, SpotLight, SunLight};
pub use mesh::{vertex_data, Mesh, MeshBuilder, MeshHandle, VertexBuffer};
pub use mtl::{Material, MaterialDefaults, MaterialTextureSet, TextureOffset};
pub use pass::{
    get_camera, set_vertex_args, DrawFlat, DrawFlatSeparate, DrawPbm, DrawPbmSeparate, DrawShaded,
    DrawShadedSeparate,
};
pub use pipe::{
    ColorBuffer, Data, DepthBuffer, DepthMode, Effect, EffectBuilder, Init, Meta, NewEffect,
    Pipeline, PipelineBuild, PipelineBuilder, PipelineData, PolyPipeline, PolyStage, PolyStages,
    Stage, StageBuilder, Target, TargetBuilder, Targets,
};
pub use renderer::Renderer;
pub use resources::{AmbientColor, ScreenDimensions, WindowMessages};
pub use shape::{InternalShape, Shape, ShapePrefab, ShapeUpload};
pub use skinning::{
    AnimatedComboMeshCreator, AnimatedVertexBufferCombination, JointIds, JointTransforms,
    JointTransformsPrefab, JointWeights,
};
pub use sprite::{Sprite, SpriteRenderData, SpriteSheet, SpriteSheetHandle, WithSpriteRender};
pub use system::RenderSystem;
pub use tex::{Texture, TextureBuilder, TextureHandle};
pub use transparent::{
    Blend, BlendChannel, BlendValue, ColorMask, Equation, Factor, Transparent, ALPHA, REPLACE,
};
pub use types::{Encoder, Factory, PipelineState, Resources};
pub use vertex::{
    Attribute, AttributeFormat, Attributes, Color, Normal, PosColor, PosNormTangTex, PosNormTex,
    PosTex, Position, Query, Separate, Tangent, TexCoord, VertexBufferCombination, VertexFormat,
    With,
};
pub use visibility::{Visibility, VisibilitySortingSystem};

pub mod error;
pub mod mouse;
pub mod pipe;

#[macro_use]
mod macros;

mod bundle;
mod cam;
mod color;
mod config;
mod formats;
mod input;
mod light;
mod mesh;
mod mtl;
mod pass;
mod renderer;
mod resources;
mod shape;
mod skinning;
mod sprite;
mod system;
mod tex;
mod transparent;
mod types;
mod vertex;
mod visibility;
