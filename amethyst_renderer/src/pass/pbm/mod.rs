pub use self::interleaved::DrawPbm;
pub use self::separate::DrawPbmSeparate;

use gfx::traits::Pod;

mod interleaved;
mod separate;

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/basic.glsl");
static VERT_SKIN_SRC: &[u8] = include_bytes!("../shaders/vertex/skinned.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/pbm.glsl");

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 1.0]
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FragmentArgs {
    point_light_count: i32,
    directional_light_count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PointLightPod {
    position: [f32; 4],
    color: [f32; 4],
    intensity: f32,
    _pad: [f32; 3],
}

unsafe impl Pod for PointLightPod {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DirectionalLightPod {
    color: [f32; 4],
    direction: [f32; 4],
}

unsafe impl Pod for DirectionalLightPod {}
