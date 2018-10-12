//! A data parallel rendering engine developed by the [Amethyst][am] project.
//!
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! # Background
//!
//! This crate provides OpenGL graphics rendering functionality through various *rendering passes*.
//! The rendering passes may handle different domains of rendering, such as "draw game objects" vs
//! "render text"; or they can handle the same domain with different variations, such as "draw
//! objects with lighting" vs "draw objects ignoring lighting".
//!
//! ## Skinning: Interleaved Versus Separate Passes
//!
//! In an application, objects may be composed of multiple renderable entities, such as a main body
//! and separate limbs. Where the limbs join the the body, it will look more realistic if the
//! vertex positions are affected by the relative positions to the body and limb.
//!
//! This is where, for a `DrawX` pass, you will find a corresponding `DrawXSeparate` pass which
//! supports vertex skinning and joint transformations to improve the render. An exception to this
//! is the `DrawSprite` pass, which does not support joint transformations.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/master/src/renderer
//! [bk]: https://www.amethyst.rs/book/master/

#![warn(missing_docs)]
#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]

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
extern crate image;
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
pub use debug_drawing::{DebugLines, DebugLinesComponent};
pub use formats::{
    build_mesh_with_combo, create_mesh_asset, create_texture_asset, BmpFormat, ComboMeshCreator,
    GraphicsPrefab, ImageData, JpgFormat, MaterialPrefab, MeshCreator, MeshData, ObjFormat,
    PngFormat, TextureData, TextureFormat, TextureMetadata, TexturePrefab, TgaFormat,
};
pub use input::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};
pub use light::{DirectionalLight, Light, LightPrefab, PointLight, SpotLight, SunLight};
pub use mesh::{vertex_data, Mesh, MeshBuilder, MeshHandle, VertexBuffer};
pub use mtl::{Material, MaterialDefaults, MaterialTextureSet, TextureOffset};
pub use pass::{
    get_camera, set_vertex_args, DrawDebugLines, DrawFlat, DrawFlatSeparate, DrawPbm,
    DrawPbmSeparate, DrawShaded, DrawShadedSeparate, DrawSprite,
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
pub use sprite::{
    Sprite, SpriteRender, SpriteSheet, SpriteSheetHandle, SpriteSheetSet, TextureCoordinates,
};
pub use sprite_visibility::{SpriteVisibility, SpriteVisibilitySortingSystem};
pub use system::RenderSystem;
pub use tex::{
    FilterMethod, SamplerInfo, SurfaceType, Texture, TextureBuilder, TextureHandle, WrapMode
};
pub use transparent::{
    Blend, BlendChannel, BlendValue, ColorMask, Equation, Factor, Transparent, ALPHA, REPLACE,
};
pub use types::{Encoder, Factory, PipelineState, Resources};
pub use vertex::{
    Attribute, AttributeFormat, Attributes, Color, Normal, PosColor, PosColorNorm, PosNormTangTex,
    PosNormTex, PosTex, Position, Query, Separate, Tangent, TexCoord, VertexBufferCombination,
    VertexFormat, With,
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
mod debug_drawing;
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
mod sprite_visibility;
mod system;
mod tex;
mod transparent;
mod types;
mod vertex;
mod visibility;
