use std::collections::{HashMap, HashSet};

use amethyst_assets::{
    distill_importer::{ImportOp, ImportedAsset},
    error::Error,
    make_handle, AssetUuid,
};
use amethyst_rendy::{
    loaders::{load_from_linear_rgba, load_from_srgba},
    palette::{LinSrgba, Srgba},
    rendy::{
        hal,
        texture::{
            image::{load_from_image, ImageFormat as DataFormat, ImageTextureConfig, Repr},
            MipLevels, TextureBuilder,
        },
    },
    types::TextureData,
    Material,
};
use gltf::{
    buffer::Data,
    material::{AlphaMode, NormalTexture, OcclusionTexture, PbrMetallicRoughness},
    texture::Info,
};

use crate::importer::{
    images::{read_image_data, ImageFormat as ImportDataFormat},
    GltfImporterState,
};

/// load a material as an asset
pub fn load_material(
    material: &gltf::Material<'_>,
    op: &mut ImportOp,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
) -> Vec<ImportedAsset> {
    if state.material_uuids.is_none() {
        state.material_uuids = Some(HashMap::new());
    }
    if state.material_transparencies.is_none() {
        state.material_transparencies = Some(HashSet::new());
    }

    let mut assets_accumulator = Vec::new();

    let pbr = material.pbr_metallic_roughness();
    let em_factor = material.emissive_factor();
    let material_name = convert_optional_index_to_string(material.index());

    let (albedo_id, albedo_asset) = load_albedo(&pbr, buffers, state, op, material_name.clone());
    let (roughness_id, roughness_asset) =
        load_metallic_roughness(&pbr, buffers, state, op, material_name.clone());
    let (emission_id, emission_asset) = load_emission(
        &em_factor,
        material.emissive_texture(),
        buffers,
        state,
        op,
        material_name.clone(),
    );
    let (normal_id, normal_asset) = load_normal(
        material.normal_texture(),
        buffers,
        state,
        op,
        material_name.clone(),
    );
    let (occlusion_id, occlusion_asset) = load_occlusion(
        material.occlusion_texture(),
        buffers,
        state,
        op,
        material_name.clone(),
    );
    let (cavity_id, cavity_asset) = load_cavity(state, op, material_name.clone());
    let alpha_cutoff = match material.alpha_mode() {
        AlphaMode::Blend => {
            state
                .material_transparencies
                .as_mut()
                .expect("Meshes hashmap didn't work")
                .insert(material_name.clone());
            std::f32::MIN_POSITIVE
        }
        AlphaMode::Mask => material.alpha_cutoff(),
        AlphaMode::Opaque => 0.0,
    };

    assets_accumulator.push(albedo_asset);
    assets_accumulator.push(roughness_asset);
    assets_accumulator.push(emission_asset);
    assets_accumulator.push(normal_asset);
    assets_accumulator.push(occlusion_asset);
    assets_accumulator.push(cavity_asset);

    let material = Material {
        alpha_cutoff,
        albedo: make_handle(albedo_id),
        emission: make_handle(emission_id),
        normal: make_handle(normal_id),
        metallic_roughness: make_handle(roughness_id),
        ambient_occlusion: make_handle(occlusion_id),
        cavity: make_handle(cavity_id),
        uv_offset: Default::default(),
    };
    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(material_name)
        .or_insert_with(|| op.new_asset_uuid());

    assets_accumulator.push(ImportedAsset {
        id,
        search_tags: vec![],
        build_deps: vec![],
        load_deps: vec![],
        build_pipeline: None,
        asset_data: Box::new(material),
    });

    assets_accumulator
}

fn load_texture_with_factor(
    texture: Option<gltf::texture::Info<'_>>,
    factor: [f32; 4],
    buffers: &Vec<Data>,
    srgb: bool,
) -> Result<(TextureBuilder<'static>, [f32; 4]), Error> {
    match texture {
        Some(info) => {
            Ok((
                load_texture(&info.texture(), buffers, srgb)?
                    .with_mip_levels(MipLevels::GenerateAuto),
                factor,
            ))
        }
        None => {
            Ok((
                if srgb {
                    load_from_srgba(Srgba::new(factor[0], factor[1], factor[2], factor[3]))
                } else {
                    load_from_linear_rgba(LinSrgba::new(factor[0], factor[1], factor[2], factor[3]))
                },
                [1.0, 1.0, 1.0, 1.0],
            ))
        }
    }
}

fn load_albedo(
    pbr: &PbrMetallicRoughness<'_>,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let albedo: TextureData = load_texture_with_factor(
        pbr.base_color_texture(),
        pbr.base_color_factor(),
        buffers,
        true,
    )
    .map(|(texture, _)| texture.into())
    .expect("The mapping between the TextureBuilder and TextureDate did not work");

    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_albedo", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(albedo),
        },
    )
}

fn load_metallic_roughness(
    pbr: &PbrMetallicRoughness<'_>,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let roughness: TextureData = load_texture_with_factor(
        pbr.metallic_roughness_texture(),
        [1.0, pbr.roughness_factor(), pbr.metallic_factor(), 1.0],
        buffers,
        false,
    )
    .map(|(texture, _)| texture.into())
    .expect("The mapping between the TextureBuilder and TextureDate did not work");

    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_metallic_roughness", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(roughness),
        },
    )
}

fn load_emission(
    em_factor: &[f32; 3],
    em_texture: Option<Info<'_>>,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let emission: TextureData = load_texture_with_factor(
        em_texture,
        [em_factor[0], em_factor[1], em_factor[2], 1.0],
        buffers,
        true,
    )
    .map(|(texture, _)| texture.into())
    .expect("The mapping between the TextureBuilder and TextureDate did not work");

    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_emission", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(emission),
        },
    )
}

fn load_normal(
    normal_texture: Option<NormalTexture<'_>>,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let normal: TextureData = {
        match normal_texture {
            Some(normal_texture) => {
                load_texture(&normal_texture.texture(), buffers, false)
                    .map(|data| data.into())
                    .expect("The mapping between the TextureBuilder and TextureDate did not work")
            }
            None => {
                // Default normal Texture
                load_from_linear_rgba(LinSrgba::new(0.5, 0.5, 1.0, 1.0)).into()
            }
        }
    };
    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_normal", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(normal),
        },
    )
}

fn load_occlusion(
    occlusion_texture: Option<OcclusionTexture<'_>>,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let occlusion: TextureData = {
        match occlusion_texture {
            Some(normal_texture) => {
                load_texture(&normal_texture.texture(), buffers, false)
                    .map(|data| data.into())
                    .expect("The mapping between the TextureBuilder and TextureDate did not work")
            }
            None => {
                // Default occlusion Texture
                load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0)).into()
            }
        }
    };
    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_occlusion", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(occlusion),
        },
    )
}

fn load_cavity(
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    material_name: String,
) -> (AssetUuid, ImportedAsset) {
    let cavity: TextureData = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0)).into();
    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}_cavity", material_name))
        .or_insert_with(|| op.new_asset_uuid());
    (
        id,
        ImportedAsset {
            id,
            search_tags: vec![],
            build_deps: vec![],
            load_deps: vec![],
            build_pipeline: None,
            asset_data: Box::new(cavity),
        },
    )
}

fn load_texture(
    texture: &gltf::Texture<'_>,
    buffers: &Vec<Data>,
    srgb: bool,
) -> Result<TextureBuilder<'static>, Error> {
    let (data, format) = read_image_data(&texture.source(), buffers)?;

    let metadata = ImageTextureConfig {
        repr: if srgb { Repr::Srgb } else { Repr::Unorm },
        format: match format {
            ImportDataFormat::Png => Some(DataFormat::PNG),
            ImportDataFormat::Jpeg => Some(DataFormat::JPEG),
        },
        sampler_info: load_sampler_info(&texture.sampler()),
        ..Default::default()
    };

    load_from_image(std::io::Cursor::new(&data), metadata).map_err(|_e| Error::Source)
}

fn load_sampler_info(sampler: &gltf::texture::Sampler<'_>) -> hal::image::SamplerDesc {
    use gltf::texture::{MagFilter, MinFilter};
    use hal::image::{Filter, SamplerDesc};

    let mag_filter = match sampler.mag_filter() {
        Some(MagFilter::Nearest) => Filter::Nearest,
        None | Some(MagFilter::Linear) => Filter::Linear,
    };

    let (min_filter, mip_filter) = match sampler.min_filter() {
        Some(MinFilter::Nearest) | Some(MinFilter::NearestMipmapNearest) => {
            (Filter::Nearest, Filter::Nearest)
        }
        None | Some(MinFilter::Linear) | Some(MinFilter::LinearMipmapLinear) => {
            (Filter::Linear, Filter::Linear)
        }
        Some(MinFilter::NearestMipmapLinear) => (Filter::Nearest, Filter::Linear),
        Some(MinFilter::LinearMipmapNearest) => (Filter::Linear, Filter::Nearest),
    };

    let wrap_s = map_wrapping(sampler.wrap_s());
    let wrap_t = map_wrapping(sampler.wrap_t());

    let mut s = SamplerDesc::new(min_filter, wrap_s);
    s.wrap_mode = (wrap_s, wrap_t, wrap_t);
    s.mag_filter = mag_filter;
    s.mip_filter = mip_filter;
    s
}

pub fn convert_optional_index_to_string(index: Option<usize>) -> String {
    if let Some(i) = index {
        i.to_string()
    } else {
        String::from("default")
    }
}

fn map_wrapping(gltf_wrap: gltf::texture::WrappingMode) -> hal::image::WrapMode {
    match gltf_wrap {
        gltf::texture::WrappingMode::ClampToEdge => hal::image::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => hal::image::WrapMode::Mirror,
        gltf::texture::WrappingMode::Repeat => hal::image::WrapMode::Tile,
    }
}
