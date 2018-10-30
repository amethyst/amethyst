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

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))] // complex project

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate amethyst_derive;
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
extern crate ron;
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

pub use {
    bundle::RenderBundle,
    cam::{ActiveCamera, ActiveCameraPrefab, Camera, CameraPrefab, Projection},
    color::Rgba,
    config::DisplayConfig,
    debug_drawing::{DebugLines, DebugLinesComponent},
    formats::{
        build_mesh_with_combo, create_mesh_asset, create_texture_asset, BmpFormat,
        ComboMeshCreator, GraphicsPrefab, ImageData, JpgFormat, MaterialPrefab, MeshCreator,
        MeshData, ObjFormat, PngFormat, TextureData, TextureFormat, TextureMetadata, TexturePrefab,
        TgaFormat,
    },
    hidden::{Hidden, HiddenPropagate},
    hide_system::HideHierarchySystem,
    input::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    light::{DirectionalLight, Light, LightPrefab, PointLight, SpotLight, SunLight},
    mesh::{vertex_data, Mesh, MeshBuilder, MeshHandle, VertexBuffer},
    mtl::{Material, MaterialDefaults, MaterialTextureSet, TextureOffset},
    pass::{
        get_camera, set_vertex_args, DebugLinesParams, DrawDebugLines, DrawFlat, DrawFlatSeparate,
        DrawPbm, DrawPbmSeparate, DrawShaded, DrawShadedSeparate, DrawSprite,
    },
    pipe::{
        ColorBuffer, Data, DepthBuffer, DepthMode, Effect, EffectBuilder, Init, Meta, NewEffect,
        Pipeline, PipelineBuild, PipelineBuilder, PipelineData, PolyPipeline, PolyStage,
        PolyStages, Stage, StageBuilder, Target, TargetBuilder, Targets,
    },
    renderer::Renderer,
    resources::{AmbientColor, ScreenDimensions, WindowMessages},
    shape::{InternalShape, Shape, ShapePrefab, ShapeUpload},
    skinning::{
        AnimatedComboMeshCreator, AnimatedVertexBufferCombination, JointIds, JointTransforms,
        JointTransformsPrefab, JointWeights,
    },
    sprite::{
        Sprite, SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, SpriteSheetSet,
        TextureCoordinates,
    },
    sprite_visibility::{SpriteVisibility, SpriteVisibilitySortingSystem},
    system::RenderSystem,
    tex::{
        FilterMethod, SamplerInfo, SurfaceType, Texture, TextureBuilder, TextureHandle, WrapMode,
    },
    transparent::{
        Blend, BlendChannel, BlendValue, ColorMask, Equation, Factor, Transparent, ALPHA, REPLACE,
    },
    types::{Encoder, Factory, PipelineState, Resources},
    vertex::{
        Attribute, AttributeFormat, Attributes, Color, Normal, PosColor, PosColorNorm,
        PosNormTangTex, PosNormTex, PosTex, Position, Query, Separate, Tangent, TexCoord,
        VertexBufferCombination, VertexFormat, With,
    },
    visibility::{Visibility, VisibilitySortingSystem},
};

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
mod hidden;
mod hide_system;
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
