use std::sync::Arc;

use assets::Source;
use gfx::texture::SamplerInfo;
use gltf;
use gltf::material::AlphaMode;
use itertools::Itertools;
use renderer::{
    JpgFormat, MaterialPrefab, PngFormat, TextureData, TextureFormat, TextureMetadata,
    TexturePrefab,
};

use super::{get_image_data, Buffers, GltfError, ImageFormat};

// Load a single material, and transform into a format usable by the engine
pub fn load_material(
    material: &gltf::Material,
    buffers: &Buffers,
    source: Arc<Source>,
    name: &str,
) -> Result<MaterialPrefab<TextureFormat>, GltfError> {
    let mut prefab = MaterialPrefab::default();
    prefab.albedo = Some(load_texture_with_factor(
        material.pbr_metallic_roughness().base_color_texture(),
        material.pbr_metallic_roughness().base_color_factor(),
        buffers,
        source.clone(),
        name,
    ).map(|(texture, _)| TexturePrefab::Data(texture))?);
    prefab.albedo_id = material.index().map(|i| i as u64 * 100);

    let (metallic, roughness) =
        load_texture_with_factor(
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
        ).map(|(texture, factors)| deconstruct_metallic_roughness(texture, factors[0], factors[1]))
            .map(|(metallic, roughness)| {
                (
                    TexturePrefab::Data(metallic.0),
                    TexturePrefab::Data(roughness.0),
                )
            })?;
    prefab.metallic = Some(metallic);
    prefab.metallic_id = material.index().map(|i| i as u64 * 10 + 1);
    prefab.roughness = Some(roughness);
    prefab.roughness_id = material.index().map(|i| i as u64 * 10 + 2);

    let em_factor = material.emissive_factor();
    prefab.emission = Some(load_texture_with_factor(
        material.emissive_texture(),
        [em_factor[0], em_factor[1], em_factor[2], 1.0],
        buffers,
        source.clone(),
        name,
    ).map(|(texture, _)| TexturePrefab::Data(texture))?);
    prefab.emission_id = material.index().map(|i| i as u64 * 10 + 3);

    // Can't use map/and_then because of Result returning from the load_texture function
    prefab.normal = match material.normal_texture() {
        Some(normal_texture) => {
            Some(
                load_texture(&normal_texture.texture(), buffers, source.clone(), name)
                    .map(|data| TexturePrefab::Data(data))?,
            )
        }

        None => None,
    };
    prefab.normal_id = material.index().map(|i| i as u64 * 10 + 4);

    // Can't use map/and_then because of Result returning from the load_texture function
    prefab.ambient_occlusion = match material.occlusion_texture() {
        Some(occlusion_texture) => {
            Some(
                load_texture(&occlusion_texture.texture(), buffers, source.clone(), name)
                    .map(|data| TexturePrefab::Data(data))?,
            )
        }

        None => None,
    };
    prefab.ambient_occlusion_id = material.index().map(|i| i as u64 * 10 + 5);
    prefab.transparent = if let AlphaMode::Blend = material.alpha_mode() {
        true
    } else {
        false
    };
    if let AlphaMode::Mask = material.alpha_mode() {
        prefab.alpha_cutoff = material.alpha_cutoff();
    }

    Ok(prefab)
}

fn deconstruct_metallic_roughness(
    data: TextureData,
    metallic_factor: f32,
    roughness_factor: f32,
) -> ((TextureData, f32), (TextureData, f32)) {
    (
        (
            deconstruct_image(&data, 2, 4), // metallic from B channel
            metallic_factor,
        ),
        (
            deconstruct_image(&data, 1, 4), // roughness from G channel
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
