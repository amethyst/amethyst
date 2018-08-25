pub use self::interleaved::DrawSprite;

mod interleaved;

use pass::util::TextureType;

use gfx::format::{ChannelType, Format, SurfaceType};
use gfx::pso::buffer::Element;
use gfx::traits::Pod;
use vertex::{Attribute, AttributeFormat, Attributes, VertexFormat, With};

static VERT_SRC: &[u8] = include_bytes!("../shaders/vertex/sprite.glsl");
static FRAG_SRC: &[u8] = include_bytes!("../shaders/fragment/sprite.glsl");

static TEXTURES: [TextureType; 1] = [TextureType::Albedo];

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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct SpriteInstance {
    pub dir_x: [f32; 2],
    pub dir_y: [f32; 2],
    pub pos: [f32; 2],
    pub u_offset: [f32; 2],
    pub v_offset: [f32; 2],
}

unsafe impl Pod for SpriteInstance {}

impl VertexFormat for SpriteInstance {
    const ATTRIBUTES: Attributes<'static> = &[
        (DirX::NAME, <Self as With<DirX>>::FORMAT),
        (DirY::NAME, <Self as With<DirY>>::FORMAT),
        (Pos::NAME, <Self as With<Pos>>::FORMAT),
        (OffsetU::NAME, <Self as With<OffsetU>>::FORMAT),
        (OffsetV::NAME, <Self as With<OffsetV>>::FORMAT),
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
