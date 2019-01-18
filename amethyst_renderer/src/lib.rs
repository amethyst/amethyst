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
//! is the `DrawFlat2D` pass, which does not support joint transformations.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/master/src/renderer
//! [bk]: https://www.amethyst.rs/book/master/

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use crate::{
    blink::{Blink, BlinkSystem},
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
    mtl::{Material, MaterialDefaults, TextureOffset},
    pass::{
        get_camera, set_vertex_args, DebugLinesParams, DrawDebugLines, DrawFlat, DrawFlat2D,
        DrawFlatSeparate, DrawPbm, DrawPbmSeparate, DrawShaded, DrawShadedSeparate, DrawSkybox,
        SkyboxColor,
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
        Flipped, Sprite, SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle,
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

mod blink;
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
