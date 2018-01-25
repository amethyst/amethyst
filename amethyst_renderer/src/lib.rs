//! A data parallel rendering engine developed by the [Amethyst][am] project.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/develop/src/renderer
//! [bk]: https://www.amethyst.rs/book/

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate gfx;
extern crate gfx_core;
#[macro_use]
extern crate gfx_macros;
extern crate hetseq;
extern crate imagefmt;
#[macro_use]
extern crate log;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate shred;
extern crate shrev;
extern crate smallvec;
extern crate specs;
extern crate wavefront_obj;
extern crate winit;

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
pub use cam::{ActiveCamera, Camera, Projection};
pub use color::Rgba;
pub use config::DisplayConfig;
pub use formats::{build_mesh_with_combo, create_mesh_asset, create_texture_asset, BmpFormat,
                  ComboMeshCreator, ImageData, ImageError, JpgFormat, MeshCreator, MeshData,
                  ObjFormat, PngFormat, TextureData, TextureMetadata};
pub use input::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
pub use light::{DirectionalLight, Light, PointLight, SpotLight, SunLight};
pub use mesh::{vertex_data, Mesh, MeshHandle, VertexBuffer};
pub use mtl::{Material, MaterialDefaults};
pub use pass::{DrawFlat, DrawFlatSeparate, DrawPbm, DrawPbmSeparate, DrawShaded,
               DrawShadedSeparate};
pub use pipe::{ColorBuffer, Data, DepthBuffer, DepthMode, Effect, EffectBuilder, Init, Meta,
               NewEffect, Pipeline, PipelineBuild, PipelineBuilder, PipelineData, PolyPipeline,
               PolyStage, PolyStages, Stage, StageBuilder, Target, TargetBuilder, Targets};
pub use renderer::Renderer;
pub use resources::{AmbientColor, ScreenDimensions, WindowMessages};
pub use skinning::{AnimatedComboMeshCreator, AnimatedVertexBufferCombination, JointIds,
                   JointTransforms, JointWeights};
pub use system::RenderSystem;
pub use tex::{Texture, TextureBuilder, TextureHandle};
pub use types::{Encoder, Factory, PipelineState, Resources};
pub use vertex::{Attribute, AttributeFormat, Attributes, Color, Normal, PosColor, PosNormTangTex,
                 PosNormTex, PosTex, Position, Query, Separate, Tangent, TexCoord,
                 VertexBufferCombination, VertexFormat, With};

pub mod error;
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
mod skinning;
mod system;
mod tex;
mod types;
mod vertex;
