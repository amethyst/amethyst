//! Contains common types that can be glob-imported (`*`) for convenience.

pub use gfx::traits::Pod;

pub use Renderer;
pub use cam::{Camera, Projection};
pub use light::*;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::Material;
pub use pipe::{Pipeline, PipelineBuilder, PolyPipeline, PolyStage, Stage, StageBuilder, Target};
pub use tex::{Texture, TextureBuilder};
pub use pass;
pub use vertex::{PosColor, PosNormTangTex, PosNormTex, PosTex, VertexFormat};
