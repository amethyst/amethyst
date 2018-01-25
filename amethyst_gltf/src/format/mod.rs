//! GLTF format

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::mem;
use std::sync::Arc;

use self::importer::{get_image_data, import, Buffers, ImageFormat};
use animation::{AnimationOutput, InterpolationType, Sampler};
use assets::{Error as AssetError, Format, FormatValue, Result as AssetResult, ResultExt, Source};
use core::cgmath::{Matrix4, SquareMatrix};
use core::transform::LocalTransform;
use gfx::Primitive;
use gfx::texture::SamplerInfo;
use gltf;
use gltf::Gltf;
use gltf_utils::AccessorIter;
use itertools::Itertools;
use renderer::{Color, JointIds, JointWeights, JpgFormat, Normal, PngFormat, Position, Separate,
               Tangent, TexCoord, TextureMetadata};

use super::*;

mod importer;

/// Gltf scene format, will cause the whole default scene to be loaded from the given file.
///
/// Using the `GltfSceneLoaderSystem` a `Handle<GltfSceneAsset>` from this format can be attached
/// to an entity in ECS, and the system will then load the full scene using the given entity
/// as the root node of the scene hierarchy.
pub struct GltfSceneFormat;

/// Format errors
#[derive(Debug)]
pub enum GltfError {
    /// Importer failed to load the json file
    GltfImporterError(self::importer::Error),

    /// GLTF have no default scene and the number of scenes is not 1
    InvalidSceneGltf(usize),

    /// GLTF primitive use a primitive type not support by gfx
    PrimitiveMissingInGfx(String),

    /// GLTF primitive missing positions
    MissingPositions,

    /// External file failed loading
    Asset(AssetError),

    /// Not implemented yet
    NotImplemented,
}

impl StdError for GltfError {
    fn description(&self) -> &str {
        use self::GltfError::*;
        match *self {
            GltfImporterError(_) => "Gltf import error",
            InvalidSceneGltf(_) => "Gltf has no default scene, and the number of scenes is not 1",
            PrimitiveMissingInGfx(_) => "Primitive missing in gfx",
            MissingPositions => "Primitive missing positions",
            Asset(_) => "File loading error",
            NotImplemented => "Not implemented",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            GltfError::GltfImporterError(ref err) => Some(err),
            GltfError::Asset(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for GltfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::GltfError::*;
        use std::error::Error;
        match *self {
            GltfImporterError(ref err) => {
                write!(f, "{}: {}", self.description(), err.description())
            }
            PrimitiveMissingInGfx(ref err) => write!(f, "{}: {}", self.description(), err),
            Asset(ref err) => write!(f, "{}: {}", self.description(), err.description()),
            InvalidSceneGltf(size) => write!(f, "{}: {}", self.description(), size),
            MissingPositions | NotImplemented => write!(f, "{}", self.description()),
        }
    }
}

impl From<self::importer::Error> for GltfError {
    fn from(err: self::importer::Error) -> Self {
        GltfError::GltfImporterError(err)
    }
}

impl From<AssetError> for GltfError {
    fn from(err: AssetError) -> Self {
        GltfError::Asset(err)
    }
}

impl Format<GltfSceneAsset> for GltfSceneFormat {
    const NAME: &'static str = "GLTFScene";

    type Options = GltfSceneOptions;

    fn import(
        &self,
        name: String,
        source: Arc<Source>,
        options: GltfSceneOptions,
        _create_reload: bool,
    ) -> AssetResult<FormatValue<GltfSceneAsset>> {
        let gltf = load_gltf(source, &name, options).chain_err(|| "Failed to import gltf scene")?;
        if gltf.default_scene.is_some() || gltf.scenes.len() == 1 {
            Ok(FormatValue::data(gltf)) // TODO: create `Reload` object
        } else {
            Err(GltfError::InvalidSceneGltf(gltf.scenes.len())).chain_err(|| "Invalid GLTF scene")
        }
    }
}

fn load_gltf(
    source: Arc<Source>,
    name: &str,
    options: GltfSceneOptions,
) -> Result<GltfSceneAsset, GltfError> {
    debug!("Loading GLTF scene {}", name);
    import(source.clone(), name)
        .map_err(GltfError::GltfImporterError)
        .and_then(|(gltf, buffers)| load_data(&gltf, &buffers, &options, source, name))
}

fn load_data(
    gltf: &Gltf,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<Source>,
    name: &str,
) -> Result<GltfSceneAsset, GltfError> {
    // TODO: morph targets, cameras
    // TODO: KHR_materials_common extension
    debug!("Loading nodes");
    let nodes = load_nodes(gltf, buffers, options)?;
    debug!("Loading scenes");
    let scenes = gltf.scenes()
        .map(|ref scene| load_scene(scene))
        .collect::<Result<Vec<GltfScene>, GltfError>>()?;
    let default_scene = gltf.default_scene().map(|s| s.index());
    debug!("Loading materials");
    let materials = gltf.materials()
        .map(|ref m| load_material(m, buffers, source.clone(), name))
        .collect::<Result<Vec<GltfMaterial>, GltfError>>()?;
    debug!("Loading animations");
    let animations = if options.load_animations {
        gltf.animations()
            .map(|ref animation| load_animation(animation, buffers))
            .collect::<Result<Vec<GltfAnimation>, GltfError>>()?
    } else {
        Vec::default()
    };
    debug!("Loading skins");
    let skins = load_skins(gltf, buffers)?;

    Ok(GltfSceneAsset {
        nodes,
        scenes,
        materials,
        animations,
        default_scene,
        options: options.clone(),
        skins,
    })
}

fn load_animation(
    animation: &gltf::Animation,
    buffers: &Buffers,
) -> Result<GltfAnimation, GltfError> {
    let (nodes, samplers) = animation
        .channels()
        .map(|ref channel| load_channel(channel, buffers))
        .collect::<Result<Vec<(usize, Sampler)>, GltfError>>()?
        .into_iter()
        .unzip();
    Ok(GltfAnimation {
        nodes,
        samplers,
        handle: None,
    })
}

fn load_channel(
    channel: &gltf::animation::Channel,
    buffers: &Buffers,
) -> Result<(usize, Sampler), GltfError> {
    use gltf::animation::TrsProperty::*;
    use gltf_utils::AccessorIter;
    let sampler = channel.sampler();
    let target = channel.target();
    let input = gltf_utils::AccessorIter::new(sampler.input(), buffers).collect::<Vec<f32>>();
    let node_index = target.node().index();
    let ty = map_interpolation_type(&sampler.interpolation());

    match target.path() {
        Translation => {
            let output = AccessorIter::new(sampler.output(), buffers).collect::<Vec<[f32; 3]>>();
            Ok((
                node_index,
                Sampler {
                    input,
                    ty,
                    output: AnimationOutput::Translation(output),
                },
            ))
        }
        Scale => {
            let output = AccessorIter::new(sampler.output(), buffers).collect::<Vec<[f32; 3]>>();
            Ok((
                node_index,
                Sampler {
                    input,
                    ty,
                    output: AnimationOutput::Scale(output),
                },
            ))
        }
        Rotation => {
            // gltf quat format: [x, y, z, w], our quat format: [w, x, y, z]
            let output = AccessorIter::<[f32; 4]>::new(sampler.output(), buffers)
                .map(|q| [q[3], q[0], q[1], q[2]])
                .collect::<Vec<_>>();
            let ty = if ty == InterpolationType::Linear {
                InterpolationType::SphericalLinear
            } else {
                ty
            };
            Ok((
                node_index,
                Sampler {
                    input,
                    ty,
                    output: AnimationOutput::Rotation(output),
                },
            ))
        }
        Weights => Err(GltfError::NotImplemented),
    }
}

fn map_interpolation_type(ty: &gltf::animation::InterpolationAlgorithm) -> InterpolationType {
    use gltf::animation::InterpolationAlgorithm::*;

    match *ty {
        Linear => InterpolationType::Linear,
        Step => InterpolationType::Step,
        CubicSpline => InterpolationType::CubicSpline,
        CatmullRomSpline => InterpolationType::CatmullRomSpline,
    }
}

// Load a single material, and transform into a format usable by the engine
fn load_material(
    material: &gltf::Material,
    buffers: &Buffers,
    source: Arc<Source>,
    name: &str,
) -> Result<GltfMaterial, GltfError> {
    let base_color = load_texture_with_factor(
        material.pbr_metallic_roughness().base_color_texture(),
        material.pbr_metallic_roughness().base_color_factor(),
        buffers,
        source.clone(),
        name,
    ).map(|(texture, factor)| (GltfTexture::new(texture), factor))?;

    let (metallic, roughness) = load_texture_with_factor(
        material
            .pbr_metallic_roughness()
            .metallic_roughness_texture(),
        [
            material.pbr_metallic_roughness().metallic_factor(),
            material.pbr_metallic_roughness().roughness_factor(),
            1.0,
            1.0,
        ],
        buffers,
        source.clone(),
        name,
    ).map(|(texture, factors)| {
        deconstruct_metallic_roughness(texture, factors[0], factors[1])
    })?;

    let double_sided = material.double_sided();
    let alpha = (
        match material.alpha_mode() {
            gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
            gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        },
        material.alpha_cutoff(),
    );

    let em_factor = material.emissive_factor();
    let emissive =
        load_texture_with_factor(
            material.emissive_texture(),
            [em_factor[0], em_factor[1], em_factor[2], 1.0],
            buffers,
            source.clone(),
            name,
        ).map(|(texture, factor)| (GltfTexture::new(texture), [factor[0], factor[1], factor[2]]))?;

    // Can't use map/and_then because of Result returning from the load_texture function
    let normal = match material.normal_texture() {
        Some(normal_texture) => Some((
            GltfTexture::new(load_texture(
                &normal_texture.texture(),
                buffers,
                source.clone(),
                name,
            )?),
            normal_texture.scale(),
        )),

        None => None,
    };

    // Can't use map/and_then because of Result returning from the load_texture function
    let occlusion = match material.occlusion_texture() {
        Some(occlusion_texture) => Some((
            GltfTexture::new(load_texture(
                &occlusion_texture.texture(),
                buffers,
                source.clone(),
                name,
            )?),
            occlusion_texture.strength(),
        )),

        None => None,
    };

    Ok(GltfMaterial {
        base_color,
        metallic,
        roughness,
        normal,
        occlusion,
        emissive,
        alpha,
        double_sided,
    })
}

fn deconstruct_metallic_roughness(
    data: TextureData,
    metallic_factor: f32,
    roughness_factor: f32,
) -> ((GltfTexture, f32), (GltfTexture, f32)) {
    (
        (
            GltfTexture::new(deconstruct_image(&data, 2, 4)), // metallic from B channel
            metallic_factor,
        ),
        (
            GltfTexture::new(deconstruct_image(&data, 1, 4)), // roughness from G channel
            roughness_factor,
        ),
    )
}

fn deconstruct_image(data: &TextureData, offset: usize, step: usize) -> TextureData {
    use gfx::format::SurfaceType;
    match *data {
        TextureData::Image(ref image_data, ref metadata) => {
            let metadata = metadata
                .clone()
                .with_size(image_data.raw.w as u16, image_data.raw.h as u16)
                .with_format(SurfaceType::R8);
            let image_data = image_data
                .raw
                .buf
                .iter()
                .dropping(offset)
                .step(step)
                .cloned()
                .collect();
            TextureData::U8(image_data, metadata)
        }
        TextureData::Rgba(ref color, ref metadata) => {
            TextureData::Rgba([color[offset]; 4], metadata.clone())
        }
        _ => unreachable!(), // We only support color and image for textures from gltf files
    }
}

fn load_texture_with_factor(
    texture: Option<gltf::texture::Info>,
    factor: [f32; 4],
    buffers: &Buffers,
    source: Arc<Source>,
    name: &str,
) -> Result<(TextureData, [f32; 4]), GltfError> {
    match texture {
        Some(info) => Ok((
            load_texture(&info.texture(), buffers, source, name)?,
            factor,
        )),
        None => Ok((TextureData::color(factor), [1.0, 1.0, 1.0, 1.0])),
    }
}

fn load_texture(
    texture: &gltf::Texture,
    buffers: &Buffers,
    source: Arc<Source>,
    name: &str,
) -> Result<TextureData, GltfError> {
    let (data, format) = get_image_data(&texture.source(), buffers, source, name.as_ref())?;
    let metadata = TextureMetadata::default().with_sampler(load_sampler_info(&texture.sampler()));
    Ok(match format {
        ImageFormat::Png => PngFormat.from_data(data, metadata),
        ImageFormat::Jpeg => JpgFormat.from_data(data, metadata),
    }?)
}

fn load_sampler_info(sampler: &gltf::texture::Sampler) -> SamplerInfo {
    use gfx::texture::{FilterMethod, WrapMode};
    use gltf::texture::{MagFilter, WrappingMode};
    // gfx only have support for a single filter, therefore we use mag filter, and ignore min filter
    let filter = match sampler.mag_filter() {
        None | Some(MagFilter::Nearest) => FilterMethod::Scale,
        Some(MagFilter::Linear) => FilterMethod::Bilinear,
    };
    let wrap_s = match sampler.wrap_s() {
        WrappingMode::ClampToEdge => WrapMode::Clamp,
        WrappingMode::MirroredRepeat => WrapMode::Mirror,
        WrappingMode::Repeat => WrapMode::Tile,
    };
    let wrap_t = match sampler.wrap_t() {
        WrappingMode::ClampToEdge => WrapMode::Clamp,
        WrappingMode::MirroredRepeat => WrapMode::Mirror,
        WrappingMode::Repeat => WrapMode::Tile,
    };
    let mut s = SamplerInfo::new(filter, wrap_s);
    s.wrap_mode.1 = wrap_t;
    s
}

fn load_scene(scene: &gltf::Scene) -> Result<GltfScene, GltfError> {
    Ok(GltfScene {
        root_nodes: scene.nodes().map(|n| n.index()).collect(),
    })
}

fn load_nodes(
    gltf: &gltf::Gltf,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<GltfNode>, GltfError> {
    let mut node_map = HashMap::default();
    let mut nodes = vec![];

    for node in gltf.nodes() {
        let node_index = nodes.len();
        let node = load_node(&node, buffers, node_index, &mut node_map, options)?;
        nodes.push(node);
    }

    for (node_index, node) in nodes.iter_mut().enumerate() {
        match node_map.get(&node_index) {
            Some(parent_index) => node.parent = Some(*parent_index),
            _ => (),
        }
    }

    Ok(nodes)
}

fn load_node(
    node: &gltf::Node,
    buffers: &Buffers,
    node_index: usize,
    node_map: &mut HashMap<usize, usize>,
    options: &GltfSceneOptions,
) -> Result<GltfNode, GltfError> {
    let children = node.children().map(|c| c.index()).collect::<Vec<_>>();

    for child in node.children() {
        node_map.insert(child.index(), node_index);
    }

    let primitives = match node.mesh() {
        Some(mesh) => match load_mesh(&mesh, buffers, options) {
            Err(err) => return Err(err),
            Ok(primitives) => primitives,
        },
        None => Vec::default(),
    };

    let (translation, rotation, scale) = node.transform().decomposed();
    let mut local_transform = LocalTransform::default();
    local_transform.translation = translation.into();
    // gltf quat format: [x, y, z, w], our quat format: [w, x, y, z]
    local_transform.rotation = [rotation[3], rotation[0], rotation[1], rotation[2]].into();
    local_transform.scale = scale.into();

    let skin = node.skin().map(|s| s.index());

    Ok(GltfNode {
        primitives,
        children,
        parent: None,
        local_transform,
        skin,
    })
}

fn load_skins(gltf: &gltf::Gltf, buffers: &Buffers) -> Result<Vec<GltfSkin>, GltfError> {
    gltf.skins().map(|s| load_skin(&s, buffers)).collect()
}

fn load_skin(skin: &gltf::Skin, buffers: &Buffers) -> Result<GltfSkin, GltfError> {
    let joints = skin.joints().map(|j| j.index()).collect::<Vec<_>>();
    let skeleton = skin.skeleton().map(|s| s.index());
    let inverse_bind_matrices = skin.inverse_bind_matrices()
        .map(|acc| AccessorIter::<[f32; 16]>::new(acc, buffers))
        .map(|matrices| {
            matrices
                .map(|m| unsafe { mem::transmute::<[f32; 16], [[f32; 4]; 4]>(m) })
                .collect::<Vec<_>>()
        })
        .unwrap_or(vec![Matrix4::identity().into(); joints.len()]);

    Ok(GltfSkin {
        joints,
        skeleton,
        inverse_bind_matrices,
    })
}

fn flip_check(uv: [f32; 2], flip_v: bool) -> [f32; 2] {
    if flip_v {
        [uv[0], 1. - uv[1]]
    } else {
        uv
    }
}

fn load_mesh(
    mesh: &gltf::Mesh,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<GltfPrimitive>, GltfError> {
    // TODO: simplify loading here when we have support for indexed meshes
    // All attributes can then be mapped directly instead of using faces to unwind the indexing
    use gltf_utils::PrimitiveIterators;

    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        let faces = primitive.indices_u32(buffers).map(|mut iter| {
            let mut faces = vec![];
            while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
                faces.push(a as usize);
                faces.push(b as usize);
                faces.push(c as usize);
            }
            faces
        });

        let positions = primitive
            .positions(buffers)
            .map(|positions| match faces {
                Some(ref faces) => {
                    let vertices = positions.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<Position>::new(vertices[*i]))
                        .collect::<Vec<_>>()
                }
                None => positions
                    .map(|pos| Separate::<Position>::new(pos))
                    .collect(),
            })
            .ok_or(GltfError::MissingPositions)?;
        let bounds = primitive.position_bounds().unwrap();

        let colors = primitive
            .colors_rgba_f32(0, 1., buffers)
            .map(|colors| match faces {
                Some(ref faces) => {
                    let colors = colors.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<Color>::new(colors[*i]))
                        .collect()
                }
                None => colors.map(|color| Separate::<Color>::new(color)).collect(),
            });

        let tex_coord = match primitive.tex_coords_f32(0, buffers) {
            Some(tex_coords) => Some(tex_coords.collect::<Vec<[f32; 2]>>()),
            None => match options.generate_tex_coords {
                Some((u, v)) => Some((0..positions.len()).map(|_| [u, v]).collect()),
                None => None,
            },
        }.map(|texs| match faces {
            Some(ref faces) => faces
                .iter()
                .map(|i| Separate::<TexCoord>::new(flip_check(texs[*i], options.flip_v_coord)))
                .collect(),
            None => texs.into_iter()
                .map(|t| Separate::<TexCoord>::new(flip_check(t, options.flip_v_coord)))
                .collect(),
        });

        let normals = primitive.normals(buffers).map(|normals| match faces {
            Some(ref faces) => {
                let normals = normals.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| Separate::<Normal>::new(normals[*i]))
                    .collect()
            }
            None => normals.map(|n| Separate::<Normal>::new(n)).collect(),
        });

        let tangents = primitive.tangents(buffers).map(|tangents| match faces {
            Some(ref faces) => {
                let tangents = tangents.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| {
                        Separate::<Tangent>::new([
                            tangents[*i][0],
                            tangents[*i][1],
                            tangents[*i][2],
                        ])
                    })
                    .collect()
            }
            None => tangents
                .map(|t| Separate::<Tangent>::new([t[0], t[1], t[2]]))
                .collect(),
        });

        let joint_ids = primitive.joints_u16(0, buffers).map(|joints| match faces {
            Some(ref faces) => {
                let joints = joints.collect::<Vec<_>>();
                faces
                    .iter()
                    .map(|i| Separate::<JointIds>::new(joints[*i]))
                    .collect()
            }
            None => joints.map(|j| Separate::<JointIds>::new(j)).collect(),
        });
        trace!("Joint ids: {:?}", joint_ids);

        let joint_weights = primitive
            .weights_f32(0, buffers)
            .map(|weights| match faces {
                Some(ref faces) => {
                    let weights = weights.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| Separate::<JointWeights>::new(weights[*i]))
                        .collect()
                }
                None => weights.map(|w| Separate::<JointWeights>::new(w)).collect(),
            });
        trace!("Joint weights: {:?}", joint_weights);

        let material = primitive.material().index();

        match map_mode(primitive.mode()) {
            Ok(primitive) => primitives.push(GltfPrimitive {
                extents: bounds.min..bounds.max,
                primitive,
                indices: faces,
                material,
                attributes: (
                    positions,
                    colors,
                    tex_coord,
                    normals,
                    tangents,
                    joint_ids,
                    joint_weights,
                ),
                handle: None,
            }),
            Err(err) => return Err(err),
        }
    }
    Ok(primitives)
}

fn map_mode(mode: gltf::mesh::Mode) -> Result<Primitive, GltfError> {
    use gltf::mesh::Mode::*;
    match mode {
        Points => Ok(Primitive::PointList),
        Lines => Ok(Primitive::LineList),
        LineLoop => Err(GltfError::PrimitiveMissingInGfx("LineLoop".to_string())),
        LineStrip => Ok(Primitive::LineStrip),
        Triangles => Ok(Primitive::TriangleList),
        TriangleStrip => Ok(Primitive::TriangleStrip),
        TriangleFan => Err(GltfError::PrimitiveMissingInGfx("TriangleFan".to_string())),
    }
}
