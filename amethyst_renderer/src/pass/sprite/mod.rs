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
pub enum Size {}
impl Attribute for Size {
    const NAME: &'static str = "size";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
pub enum Offset {}
impl Attribute for Offset {
    const NAME: &'static str = "offset";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
pub enum OffsetU {}
impl Attribute for OffsetU {
    const NAME: &'static str = "u_offset";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[derive(Clone, Debug)]
pub enum OffsetV {}
impl Attribute for OffsetV {
    const NAME: &'static str = "v_offset";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteInstance {
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub u_offset: [f32; 2],
    pub v_offset: [f32; 2],
}

unsafe impl Pod for SpriteInstance {}

impl VertexFormat for SpriteInstance {
    const ATTRIBUTES: Attributes<'static> = &[
        (Size::NAME, <Self as With<Size>>::FORMAT),
        (Offset::NAME, <Self as With<Offset>>::FORMAT),
        (OffsetU::NAME, <Self as With<OffsetU>>::FORMAT),
        (OffsetV::NAME, <Self as With<OffsetV>>::FORMAT),
    ];
}

impl With<Size> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Size::FORMAT,
    };
}

impl With<Offset> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: Size::SIZE,
        format: Offset::FORMAT,
    };
}

impl With<OffsetU> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: Size::SIZE + Offset::SIZE,
        format: OffsetU::FORMAT,
    };
}

impl With<OffsetV> for SpriteInstance {
    const FORMAT: AttributeFormat = Element {
        offset: Size::SIZE + Offset::SIZE + OffsetU::SIZE,
        format: OffsetV::FORMAT,
    };
}
