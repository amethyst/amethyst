pub use self::interleaved::DrawSkybox;

use serde::{Deserialize, Serialize};

use crate::color::Rgba;

mod interleaved;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/skybox.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/skybox.glsl");

/// Colors used for the gradient skybox
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkyboxColor {
    /// The color directly above the viewer
    pub zenith: Rgba,
    /// The color directly below the viewer
    pub nadir: Rgba,
}

impl Default for SkyboxColor {
    fn default() -> SkyboxColor {
        SkyboxColor {
            zenith: Rgba(0.75, 1.0, 1.0, 1.0),
            nadir: Rgba(0.1, 0.3, 0.35, 1.0),
        }
    }
}
