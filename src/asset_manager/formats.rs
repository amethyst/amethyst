//! The asset formats module, which declares structs for the supported
//! file formats. You may add such a format yourself, by implementing
//! `AssetFormat` and `Import` for a struct. For an example, see
//! the documentation of the `Asset` trait.

use asset_manager::{AssetFormat, Import, ImportError};

use std::io::Cursor;

// -----------------------------------------------------------------------------------------
// Image formats
// -----------------------------------------------------------------------------------------

use imagefmt;
use dds::{Header, DDS};

use ecs::components::{TextureData, TextureLoadData};
use gfx::texture::{Kind, AaMode};

macro_rules! replace_expr {
    ($_t:tt, $with:expr) => {$with};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts, 1usize))*};
}

macro_rules! string_array {
    ( $( $x:expr ),* ) => (
        {
            const NUM_ARGS: usize = count_tts!( $( ($x) )* );
            const SA: [&'static str; NUM_ARGS] = [$( $x, )*];
            const SA_REF: &'static [&'static str; NUM_ARGS] = &SA;

            SA_REF
        }
    );
}

/// Docs
pub struct Png;

/// Docs
pub struct Jpg;

/// Docs
pub struct Tga;

/// Docs
pub struct Bmp;

/// Docs
pub struct Dds;

fn read_image<F>(bytes: Box<[u8]>, f: F) -> Result<TextureData, ImportError>
    where F: FnOnce(&mut Cursor<&[u8]>, imagefmt::ColFmt) -> imagefmt::Result<imagefmt::Image<u8>>
{
    let mut bytes_ref = Cursor::new(bytes.as_ref());

    let data = f(&mut bytes_ref, imagefmt::ColFmt::RGBA)
            .map_err(|x| ImportError::FormatError(format!("Failed to load png: {}", x)))?;

    let (w, h, pixels): (u16, u16, Vec<u8>) = (data.w as u16, data.h as u16, data.buf);

    Ok(TextureData::Raw(TextureLoadData {
        kind: Kind::D2(w, h, AaMode::Single),
        raw: Box::new([pixels.chunks(4)
                           .map(|x| [x[0], x[1], x[2], x[3]])
                           .collect::<Vec<_>>()
                           .into_boxed_slice()]),
    }))
}

impl AssetFormat for Png {
    fn file_extensions(&self) -> &[&str] {
        string_array!("png")
    }
}

impl Import<TextureData> for Png {
    fn import(&self, bytes: Box<[u8]>) -> Result<TextureData, ImportError> {
        read_image(bytes, |c, f| imagefmt::png::read(c, f))
    }
}

impl AssetFormat for Jpg {
    fn file_extensions(&self) -> &[&str] {
        string_array!("jpg")
    }
}

impl Import<TextureData> for Jpg {
    fn import(&self, bytes: Box<[u8]>) -> Result<TextureData, ImportError> {
        read_image(bytes, |c, f| imagefmt::jpeg::read(c, f))
    }
}

impl AssetFormat for Tga {
    fn file_extensions(&self) -> &[&str] {
        string_array!("tga")
    }
}

impl Import<TextureData> for Tga {
    fn import(&self, bytes: Box<[u8]>) -> Result<TextureData, ImportError> {
        read_image(bytes, |c, f| imagefmt::tga::read(c, f))
    }
}

impl AssetFormat for Bmp {
    fn file_extensions(&self) -> &[&str] {
        string_array!("bmp")
    }
}

impl Import<TextureData> for Bmp {
    fn import(&self, bytes: Box<[u8]>) -> Result<TextureData, ImportError> {
        read_image(bytes, |c, f| imagefmt::bmp::read(c, f))
    }
}

impl AssetFormat for Dds {
    fn file_extensions(&self) -> &[&str] {
        string_array!("dds")
    }
}

impl Import<TextureData> for Dds {
    fn import(&self, bytes: Box<[u8]>) -> Result<TextureData, ImportError> {
        let mut bytes_ref = bytes.as_ref();

        let DDS { header: Header { width: w, height: h, .. }, layers } =
            DDS::decode(&mut bytes_ref).ok_or(ImportError::FormatError("Unsupported DDS format".to_string()))?;

        let raw =
            layers.into_iter().map(|x| x.into_boxed_slice()).collect::<Vec<_>>().into_boxed_slice();
        Ok(TextureData::Raw(TextureLoadData {
            kind: Kind::D2(w as u16, h as u16, AaMode::Single),
            raw: raw,
        }))
    }
}

use std::str;
use cgmath::{Vector3, InnerSpace};
use renderer::VertexPosNormal;

use wavefront_obj::obj::{Primitive, parse as parse_obj};

/// Wavefront OBJ model format.
pub struct Obj;

impl Import<Vec<VertexPosNormal>> for Obj {
    fn import(&self, bytes: Box<[u8]>) -> Result<Vec<VertexPosNormal>, ImportError> {

        let obj_set =
            parse_obj(str::from_utf8(bytes.as_ref())
                    .map_err(|_| ImportError::FormatError("Invalid UTF8".to_string()))?
                    .to_string()).map_err(|x| {
                    ImportError::FormatError(format!("Failed to parse OBJ file: {:?}", x))
                })?;

        // Takes a list of objects that contain geometries that contain shapes that contain
        // vertex/texture/normal indices into the main list of vertices, and converts to a
        // flat vec of `VertexPosNormal` objects.
        // TODO: Doesn't differentiate between objects in a `*.obj` file, treats
        // them all as a single mesh.
        let vertices: Vec<VertexPosNormal> = obj_set.objects
            .iter()
            .flat_map(|object| {
                object.geometry
                    .iter()
                    .flat_map(|ref geometry| {
                        geometry.shapes.iter().flat_map(|s| -> Vec<VertexPosNormal> {
                            let mut vtn_indices = vec![];

                            match s.primitive {
                                Primitive::Point(v1) => {
                                    vtn_indices.push(v1);
                                }
                                Primitive::Line(v1, v2) => {
                                    vtn_indices.push(v1);
                                    vtn_indices.push(v2);
                                }
                                Primitive::Triangle(v1, v2, v3) => {
                                    vtn_indices.push(v1);
                                    vtn_indices.push(v2);
                                    vtn_indices.push(v3);
                                }
                            }

                            vtn_indices.iter()
                                .map(|&(vi, ti, ni)| {
                                    let vertex = object.vertices[vi];

                                    VertexPosNormal {
                                        pos: [vertex.x as f32, vertex.y as f32, vertex.z as f32],
                                        normal: match ni {
                                            Some(i) => {
                                                let normal = object.normals[i];

                                                Vector3::from([normal.x as f32,
                                                               normal.y as f32,
                                                               normal.z as f32])
                                                    .normalize()
                                                    .into()
                                            }
                                            None => [0.0, 0.0, 0.0],
                                        },
                                        tex_coord: match ti {
                                            Some(i) => {
                                                let tvertex = object.tex_vertices[i];
                                                [tvertex.u as f32, tvertex.v as f32]
                                            }
                                            None => [0.0, 0.0],
                                        },
                                    }
                                })
                                .collect()
                        })
                    })
                    .collect::<Vec<VertexPosNormal>>()
            })
            .collect();

        Ok(vertices)
    }
}
