pub use self::interleaved::DrawSkybox;

use color::Rgba;

mod interleaved;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/skybox.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/skybox.glsl");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkyboxColor {
    pub zenith: Rgba,
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