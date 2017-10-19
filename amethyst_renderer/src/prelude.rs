//! Contains common types that can be glob-imported (`*`) for convenience.

pub use gfx::traits::Pod;

pub use Renderer;
pub use cam::{Camera, Projection};
pub use color::Rgba;
pub use formats::{MeshData, TextureData, TextureMetadata};
pub use input;
pub use light::*;
pub use mesh::{Mesh, MeshBuilder, MeshHandle};
pub use mtl::{Material, MaterialDefaults};
pub use pass;
pub use pipe::{Pipeline, PipelineBuilder, PolyPipeline, PolyStage, Stage, StageBuilder, Target};
pub use resources::*;
pub use system::RenderSystem;
pub use tex::{Texture, TextureBuilder, TextureHandle};
pub use vertex::{Color, Normal, PosColor, PosNormTangTex, PosNormTex, PosTex, Position, Separate,
                 Tangent, TexCoord, VertexBufferCombination, VertexFormat};
