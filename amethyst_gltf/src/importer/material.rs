use amethyst_assets::{distill_importer::{ImportedAsset, ImportOp}, error::Error, make_handle, AssetUuid};
use amethyst_rendy::{
    loaders::{load_from_linear_rgba, load_from_srgba},
    palette::{LinSrgba, Srgba},
    rendy::{
        core::hal::image::Filter,
        hal,
        texture::{
            image::{load_from_image, ImageFormat as DataFormat, ImageTextureConfig, Repr},
            MipLevels, TextureBuilder,
        },
    },
    types::TextureData,
    Material,
};
use gltf::{buffer::Data};

use crate::importer::{
    images::{read_image_data, ImageFormat as ImportDataFormat},
    GltfImporterState,
};
use std::collections::HashMap;
use gltf::material::PbrMetallicRoughness;

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

    let mut assets_accumulator = Vec::new();

    // TODO : normal (or default)
    // TODO : Ambient occlusion (or default)
    // TODO : alpha mode

    let pbr = material.pbr_metallic_roughness();
    let em_factor = material.emissive_factor();

    let (albedo_id, albedo_asset) = load_albedo(&pbr, buffers, state, op, material.index());
    let (roughness_id, roughness_asset) = load_metallic_roughness(&pbr, buffers, state, op, material.index());
    let (emission_id, emission_asset) = load_emission(&em_factor, buffers, state, op, material.index());


    assets_accumulator.push(albedo_asset);
    assets_accumulator.push(roughness_asset);
    assets_accumulator.push(emission_asset);


/*
    let mut material = Material {
        alpha_cutoff: 0.0,
        albedo: make_handle(albedo_id),
        emission: make_handle(emission_id),
        normal: (),
        metallic_roughness: make_handle(roughness_id),
        ambient_occlusion: (),
        cavity: (),
        uv_offset: Default::default(),
    };
*/
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
    pbr: &PbrMetallicRoughness,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    index: Option<usize>
) -> (AssetUuid, ImportedAsset) {
    let albedo: TextureData =
        load_texture_with_factor(
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
        .entry(format!("{}albedo", index.map(|i| i.to_string()).unwrap_or("default".to_string())))
        .or_insert_with(|| op.new_asset_uuid());
    (id,
     ImportedAsset {
         id,
         search_tags: vec![],
         build_deps: vec![],
         load_deps: vec![],
         build_pipeline: None,
         asset_data: Box::new(albedo),
     }
    )
}

fn load_metallic_roughness(
    pbr: &PbrMetallicRoughness,
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    index: Option<usize>
) -> (AssetUuid, ImportedAsset) {
    let roughness: TextureData = load_texture_with_factor(
        pbr.metallic_roughness_texture(),
        [1.0, pbr.roughness_factor(), pbr.metallic_factor(), 1.0],
        buffers,
        source.clone()
    ).map(|(texture, _)| texture.into())
        .expect("The mapping between the TextureBuilder and TextureDate did not work");

    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}metallic-roughness", index.map(|i| i.to_string()).unwrap_or("default".to_string())))
        .or_insert_with(|| op.new_asset_uuid());
    (id,
     ImportedAsset {
         id,
         search_tags: vec![],
         build_deps: vec![],
         load_deps: vec![],
         build_pipeline: None,
         asset_data: Box::new(roughness),
     }
    )
}

fn load_emission(
    em_factor: &[f32; 3],
    buffers: &Vec<Data>,
    state: &mut GltfImporterState,
    op: &mut ImportOp,
    index: Option<usize>
) -> (AssetUuid, ImportedAsset) {
    let emission: TextureData = load_texture_with_factor(
        material.emissive_texture(),
        [em_factor[0], em_factor[1], em_factor[2], 1.0],
        buffers,
        source.clone()
    ).map(|(texture, _)| texture.into())
        .expect("The mapping between the TextureBuilder and TextureDate did not work");

    let id = *state
        .material_uuids
        .as_mut()
        .expect("Meshes hashmap didn't work")
        .entry(format!("{}emission", index.map(|i| i.to_string()).unwrap_or("default".to_string())))
        .or_insert_with(|| op.new_asset_uuid());
    (id,
     ImportedAsset {
         id,
         search_tags: vec![],
         build_deps: vec![],
         load_deps: vec![],
         build_pipeline: None,
         asset_data: Box::new(emission),
     }
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

fn map_wrapping(gltf_wrap: gltf::texture::WrappingMode) -> hal::image::WrapMode {
    match gltf_wrap {
        gltf::texture::WrappingMode::ClampToEdge => hal::image::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => hal::image::WrapMode::Mirror,
        gltf::texture::WrappingMode::Repeat => hal::image::WrapMode::Tile,
    }
}
