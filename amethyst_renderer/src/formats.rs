//! Provides texture formats
//!

pub use imagefmt::Error as ImageError;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;
use std::string::FromUtf8Error;
use std::sync::Arc;

use amethyst_assets::{Asset, BoxedErr, Format, Source};
use cgmath::{InnerSpace, Vector3};
use imagefmt;
use imagefmt::{ColFmt, Image};
use gfx::format::{ChannelType, SurfaceType};
use gfx::texture::SamplerInfo;
use gfx::traits::Pod;
use wavefront_obj::ParseError;
use wavefront_obj::obj::{parse, Normal, NormalIndex, ObjSet, Object, Primitive, TVertex,
                         TextureIndex, Vertex, VertexIndex};

use mesh::Mesh;
use tex::{Texture, TextureBuilder};
use vertex::*;
use Renderer;

/// Texture metadata, used while loading
pub struct TextureMetadata {
    /// Sampler info
    pub sampler: Option<SamplerInfo>,
    /// Mipmapping
    pub mip_levels: Option<u8>,
    /// Texture size
    pub size: Option<(u16, u16)>,
    /// Dynamic texture
    pub dynamic: bool,
    /// Surface type
    pub format: Option<SurfaceType>,
    /// Channel type
    pub channel: Option<ChannelType>,
}

/// Texture data for loading
pub enum TextureData {
    /// Image data
    Image(ImageData, Option<TextureMetadata>),

    /// Color
    Rgba([f32; 4], Option<TextureMetadata>),

    /// Float data
    F32(Vec<f32>, Option<TextureMetadata>),

    /// Float data
    F64(Vec<f64>, Option<TextureMetadata>),

    /// Byte data
    U8(Vec<u8>, Option<TextureMetadata>),

    /// Byte data
    U16(Vec<u16>, Option<TextureMetadata>),

    /// Byte data
    U32(Vec<u32>, Option<TextureMetadata>),

    /// Byte data
    U64(Vec<u64>, Option<TextureMetadata>),
}

impl Asset for Texture {
    type Data = TextureData;
}

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    /// The raw image data.
    pub raw: Image<u8>,
}
/// Allows loading of jpg or jpeg files.
pub struct JpgFormat;

impl Format<Texture> for JpgFormat {
    const NAME: &'static str = "JPEG";

    fn import(&self, name: String, source: Arc<Source>) -> Result<TextureData, BoxedErr> {
        imagefmt::jpeg::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, None))
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of PNG files.
pub struct PngFormat;

impl Format<Texture> for PngFormat {
    const NAME: &'static str = "JPEG";

    fn import(&self, name: String, source: Arc<Source>) -> Result<TextureData, BoxedErr> {
        imagefmt::png::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, None))
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of BMP files.
pub struct BmpFormat;

impl Format<Texture> for BmpFormat {
    const NAME: &'static str = "BMP";

    fn import(&self, name: String, source: Arc<Source>) -> Result<TextureData, BoxedErr> {
        imagefmt::bmp::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, None))
            .map_err(BoxedErr::new)
    }
}

/// Error type of `ObjFormat`
#[derive(Debug)]
pub enum ObjError {
    /// Coundn't convert bytes to `String`
    Utf8(FromUtf8Error),
    /// Cound't parse obj file
    Parse(ParseError),
}

impl Error for ObjError {
    fn description(&self) -> &str {
        match *self {
            ObjError::Utf8(ref err) => err.description(),
            ObjError::Parse(_) => "Obj parsing error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ObjError::Utf8(ref err) => Some(err),
            ObjError::Parse(_) => None,
        }
    }
}

impl Display for ObjError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            ObjError::Utf8(ref err) => write!(fmt, "Obj file not a unicode: {:?}", err),
            ObjError::Parse(ref err) => write!(fmt, "Obj parsing error: {}", err.message),
        }
    }
}

/// Mesh data for loading
pub enum MeshData {
    /// Position and color
    PosColor(Vec<PosColor>),

    /// Position and texture coordinates
    PosTex(Vec<PosTex>),

    /// Position, normal and texture coordinates
    PosNormTex(Vec<PosNormTex>),

    /// Position, normal, tangent and texture coordinates
    PosNormTangTex(Vec<PosNormTangTex>),
}

impl From<Vec<PosColor>> for MeshData {
    fn from(data: Vec<PosColor>) -> Self {
        MeshData::PosColor(data)
    }
}

impl From<Vec<PosTex>> for MeshData {
    fn from(data: Vec<PosTex>) -> Self {
        MeshData::PosTex(data)
    }
}

impl From<Vec<PosNormTex>> for MeshData {
    fn from(data: Vec<PosNormTex>) -> Self {
        MeshData::PosNormTex(data)
    }
}

impl From<Vec<PosNormTangTex>> for MeshData {
    fn from(data: Vec<PosNormTangTex>) -> Self {
        MeshData::PosNormTangTex(data)
    }
}

impl Asset for Mesh {
    type Data = MeshData;
}

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
pub struct ObjFormat;

impl Format<Mesh> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";

    fn import(&self, name: String, source: Arc<Source>) -> Result<MeshData, BoxedErr> {
        String::from_utf8(source.load(&name)?)
            .map_err(ObjError::Utf8)
            .and_then(|string| parse(string).map_err(ObjError::Parse))
            .map(|set| from_data(set).into())
            .map_err(BoxedErr::new)
    }
}

fn convert(
    object: &Object,
    vi: VertexIndex,
    ti: Option<TextureIndex>,
    ni: Option<NormalIndex>,
) -> PosNormTex {
    PosNormTex {
        position: {
            let vertex: Vertex = object.vertices[vi];
            [vertex.x as f32, vertex.y as f32, vertex.z as f32]
        },
        normal: ni.map(|i| {
            let normal: Normal = object.normals[i];
            Vector3::from([normal.x as f32, normal.y as f32, normal.z as f32])
                .normalize()
                .into()
        }).unwrap_or([0.0, 0.0, 0.0]),
        tex_coord: ti.map(|i| {
            let tvertex: TVertex = object.tex_vertices[i];
            [tvertex.u as f32, tvertex.v as f32]
        }).unwrap_or([0.0, 0.0]),
    }
}

fn convert_primitive(object: &Object, prim: &Primitive) -> Option<[PosNormTex; 3]> {
    match *prim {
        Primitive::Triangle(v1, v2, v3) => Some([
            convert(object, v1.0, v1.1, v1.2),
            convert(object, v2.0, v2.1, v2.2),
            convert(object, v3.0, v3.1, v3.2),
        ]),
        _ => None,
    }
}

fn from_data(obj_set: ObjSet) -> Vec<PosNormTex> {
    // Takes a list of objects that contain geometries that contain shapes that contain
    // vertex/texture/normal indices into the main list of vertices, and converts to a
    // flat vec of `PosNormTex` objects.
    // TODO: Doesn't differentiate between objects in a `*.obj` file, treats
    // them all as a single mesh.
    let vertices = obj_set.objects.iter().flat_map(|object| {
        object.geometry.iter().flat_map(move |geometry| {
            geometry
                .shapes
                .iter()
                .filter_map(move |s| convert_primitive(object, &s.primitive))
        })
    });

    let mut result = Vec::new();
    for vvv in vertices {
        result.push(vvv[0]);
        result.push(vvv[1]);
        result.push(vvv[2]);
    }
    result
}

/// Create mesh
pub fn create_mesh_asset(data: MeshData, renderer: &mut Renderer) -> ::error::Result<Mesh> {
    use MeshBuilder;
    match data {
        MeshData::PosColor(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosNormTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosNormTangTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
    }
}

/// Error that can occur during texture creation
#[derive(Debug)]
pub enum TextureError {
    /// Error occured in renderer
    Renderer(::error::Error),

    /// Color format unsupported
    UnsupportedColorFormat(ColFmt),

    /// Texture is oversized
    UnsupportedSize {
        /// Maximum size of texture (width, height)
        max: (usize, usize),

        /// Image size (width, height)
        got: (usize, usize),
    },
}

impl Error for TextureError {
    fn description(&self) -> &str {
        match *self {
            TextureError::Renderer(ref err) => err.description(),
            TextureError::UnsupportedColorFormat(_) => "Unsupported color format",
            TextureError::UnsupportedSize { .. } => "Unsupported size",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TextureError::Renderer(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for TextureError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            TextureError::Renderer(ref err) => write!(fmt, "Render error: {}", err),
            TextureError::UnsupportedColorFormat(col_fmt) => {
                write!(fmt, "Unsupported color format: {:?}", col_fmt)
            }
            TextureError::UnsupportedSize { max, got } => {
                write!(fmt, "Unsupported size. max: {:?}, got: {:?}", max, got)
            }
        }
    }
}

/// Create a texture asset.
pub fn create_texture_asset(
    data: TextureData,
    renderer: &mut Renderer,
) -> Result<Texture, BoxedErr> {
    use self::TextureData::*;
    match data {
        Image(image_data, options) => {
            create_texture_asset_from_image(image_data, options, renderer)
        }

        Rgba(color, options) => {
            let tb = apply_options(Texture::from_color_val(color), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        F32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        F64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U8(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U16(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }
    }
}

fn apply_options<D, T>(
    mut tb: TextureBuilder<D, T>,
    options: Option<TextureMetadata>,
) -> TextureBuilder<D, T>
where
    D: AsRef<[T]>,
    T: Pod + Copy,
{
    match options {
        Some(metadata) => {
            match metadata.sampler {
                Some(sampler) => tb = tb.with_sampler(sampler),
                _ => (),
            }

            match metadata.mip_levels {
                Some(mip) => tb = tb.mip_levels(mip),
                _ => (),
            }

            match metadata.size {
                Some((w, h)) => tb = tb.with_size(w, h),
                _ => (),
            }

            if metadata.dynamic {
                tb = tb.dynamic(true);
            }

            match metadata.format {
                Some(format) => tb = tb.with_format(format),
                _ => (),
            }

            match metadata.channel {
                Some(channel) => tb = tb.with_channel_type(channel),
                _ => (),
            }
        }

        None => (),
    }

    tb
}

fn create_texture_asset_from_image(
    image: ImageData,
    options: Option<TextureMetadata>,
    renderer: &mut Renderer,
) -> Result<Texture, BoxedErr> {
    fn convert_color_format(fmt: ColFmt) -> Option<SurfaceType> {
        match fmt {
            ColFmt::Auto => unreachable!(),
            ColFmt::RGBA => Some(SurfaceType::R8_G8_B8_A8),
            ColFmt::BGRA => Some(SurfaceType::B8_G8_R8_A8),
            _ => None,
        }
    }

    let image = image.raw;
    let fmt = match convert_color_format(image.fmt) {
        Some(fmt) => fmt,
        None => {
            return Err(BoxedErr::new(
                TextureError::UnsupportedColorFormat(image.fmt),
            ))
        }
    };

    if image.w > u16::max_value() as usize || image.h > u16::max_value() as usize {
        return Err(BoxedErr::new(TextureError::UnsupportedSize {
            max: (u16::max_value() as usize, u16::max_value() as usize),
            got: (image.w, image.h),
        }));
    }

    let tb = apply_options(
        TextureBuilder::new(image.buf)
            .with_format(fmt)
            .with_size(image.w as u16, image.h as u16),
        options,
    );

    renderer.create_texture(tb).map_err(BoxedErr::new)
}
