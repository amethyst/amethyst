pub use self::encode_image::{Flat2DImageEncoder, RenderImageFlat2D};
pub use self::encode_sprite::{Flat2DSpriteEncoder, RenderSpriteFlat2D};
pub use self::encode_spritesheet::{Flat2DSpriteSheetEncoder, RenderSpriteSheetFlat2D};
pub use self::interleaved::DrawFlat2D;
pub use self::sorting::Flat2DDataSorter;

mod encode_image;
mod encode_sprite;
mod encode_spritesheet;
mod interleaved;
mod sorting;

use amethyst_core::nalgebra::Vector4;
use gfx::{
    format::{ChannelType, Format, SurfaceType},
    pso::buffer::Element,
    traits::Pod,
};

use crate::{
    pass::util::TextureType,
    vertex::{Attribute, AttributeFormat, Attributes, VertexFormat, With},
    Color, Rgba, TextureHandle,
};

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/sprite.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/sprite.glsl");

static TEXTURES: [TextureType; 1] = [TextureType::Albedo];

#[derive(Clone, Debug)]
pub struct Flat2DData {
    pub texture: TextureHandle,
    pub dir_x: Vector4<f32>,
    pub dir_y: Vector4<f32>,
    pub pos: Vector4<f32>,
    pub uv_left: f32,
    pub uv_right: f32,
    pub uv_top: f32,
    pub uv_bottom: f32,
    pub tint: Rgba,
    pub transparent: bool,
}
#[derive(Clone, Debug)]
enum DirX {}
impl Attribute for DirX {
    const NAME: &'static str = "dir_x";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
enum DirY {}
impl Attribute for DirY {
    const NAME: &'static str = "dir_y";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
enum Pos {}
impl Attribute for Pos {
    const NAME: &'static str = "pos";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
enum OffsetU {}
impl Attribute for OffsetU {
    const NAME: &'static str = "u_offset";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
enum OffsetV {}
impl Attribute for OffsetV {
    const NAME: &'static str = "v_offset";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
enum Depth {}
impl Attribute for Depth {
    const NAME: &'static str = "depth";
    const FORMAT: Format = Format(SurfaceType::R32, ChannelType::Float);
    const SIZE: u32 = 4;
    type Repr = f32;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct SpriteInstance {
    pub dir_x: [f32; 2],
    pub dir_y: [f32; 2],
    pub pos: [f32; 2],
    pub u_offset: [f32; 2],
    pub v_offset: [f32; 2],
    pub color: [f32; 4],
    pub depth: f32,
}

unsafe impl Pod for SpriteInstance {}

impl VertexFormat for SpriteInstance {
    const ATTRIBUTES: Attributes<'static> = &[
        (DirX::NAME, <Self as With<DirX>>::FORMAT),
        (DirY::NAME, <Self as With<DirY>>::FORMAT),
        (Pos::NAME, <Self as With<Pos>>::FORMAT),
        (OffsetU::NAME, <Self as With<OffsetU>>::FORMAT),
        (OffsetV::NAME, <Self as With<OffsetV>>::FORMAT),
        (Color::NAME, <Self as With<Color>>::FORMAT),
        (Depth::NAME, <Self as With<Depth>>::FORMAT),
    ];
}

impl With<DirX> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: DirX::FORMAT,
    };
}

impl With<DirY> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE,
        format: DirY::FORMAT,
    };
}

impl With<Pos> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE + DirY::SIZE,
        format: Pos::FORMAT,
    };
}

impl With<OffsetU> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE + DirY::SIZE + Pos::SIZE,
        format: OffsetU::FORMAT,
    };
}

impl With<OffsetV> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE + DirY::SIZE + Pos::SIZE + OffsetU::SIZE,
        format: OffsetV::FORMAT,
    };
}

impl With<Color> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE + DirY::SIZE + Pos::SIZE + OffsetU::SIZE + OffsetV::SIZE,
        format: Color::FORMAT,
    };
}

impl With<Depth> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: DirX::SIZE + DirY::SIZE + Pos::SIZE + OffsetU::SIZE + OffsetV::SIZE + Color::SIZE,
        format: Depth::FORMAT,
    };
}
