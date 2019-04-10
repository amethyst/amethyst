use super::{get_image_data, Buffers, ImageFormat as ImportDataFormat};
use amethyst_assets::Source;
use amethyst_error::Error;
use amethyst_rendy::{
    formats::{
        mtl::MaterialPrefab,
        texture::{ImageFormat, TexturePrefab},
    },
    palette::{LinSrgba, Srgba},
    rendy::{
        hal::{self, Backend},
        texture::{
            image::{load_from_image, ImageFormat as DataFormat, ImageTextureConfig, Repr},
            palette::{load_from_linear_rgba, load_from_srgba},
            TextureBuilder,
        },
    },
};

// use gfx::texture::SamplerInfo;
use gltf::{self, material::AlphaMode};
use std::sync::Arc;

// Load a single material, and transform into a format usable by the engine
pub fn load_material<B: Backend>(
    material: &gltf::Material<'_>,
    buffers: &Buffers,
    source: Arc<dyn Source>,
    name: &str,
) -> Result<MaterialPrefab<B, ImageFormat>, Error> {
    let mut prefab = MaterialPrefab::default();

    let pbr = material.pbr_metallic_roughness();

    prefab.albedo = Some(
        load_texture_with_factor(
            pbr.base_color_texture(),
            pbr.base_color_factor(),
            buffers,
            source.clone(),
            name,
            true,
        )
        .map(|(texture, _)| TexturePrefab::Data(texture))?,
    );

    // metallic from B channel
    // roughness from G channel
    let metallic_roughness = load_texture_with_factor(
        pbr.metallic_roughness_texture(),
        [1.0, pbr.roughness_factor(), pbr.metallic_factor(), 1.0],
        buffers,
        source.clone(),
        name,
        false,
    )?
    .0;

    prefab.metallic_roughness = Some(TexturePrefab::Data(metallic_roughness));

    let em_factor = material.emissive_factor();
    prefab.emission = Some(TexturePrefab::Data(
        load_texture_with_factor(
            material.emissive_texture(),
            [em_factor[0], em_factor[1], em_factor[2], 1.0],
            buffers,
            source.clone(),
            name,
            false,
        )?
        .0,
    ));

    // Can't use map/and_then because of Result returning from the load_texture function
    prefab.normal = match material.normal_texture() {
        Some(normal_texture) => Some(
            load_texture(
                &normal_texture.texture(),
                buffers,
                source.clone(),
                name,
                false,
            )
            .map(|data| TexturePrefab::Data(data))?,
        ),

        None => None,
    };

    // Can't use map/and_then because of Result returning from the load_texture function
    prefab.ambient_occlusion = match material.occlusion_texture() {
        Some(occlusion_texture) => Some(
            load_texture(
                &occlusion_texture.texture(),
                buffers,
                source.clone(),
                name,
                false,
            )
            .map(|data| TexturePrefab::Data(data))?,
        ),

        None => None,
    };
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

fn load_texture_with_factor(
    texture: Option<gltf::texture::Info<'_>>,
    factor: [f32; 4],
    buffers: &Buffers,
    source: Arc<dyn Source>,
    name: &str,
    srgb: bool,
) -> Result<(TextureBuilder<'static>, [f32; 4]), Error> {
    match texture {
        Some(info) => Ok((
            load_texture(&info.texture(), buffers, source, name, srgb)?,
            factor,
        )),
        None => Ok((
            if srgb {
                load_from_srgba(Srgba::new(factor[0], factor[1], factor[2], factor[3]))
            } else {
                load_from_linear_rgba(LinSrgba::new(factor[0], factor[1], factor[2], factor[3]))
            },
            [1.0, 1.0, 1.0, 1.0],
        )),
    }
}

fn load_texture(
    texture: &gltf::Texture<'_>,
    buffers: &Buffers,
    source: Arc<dyn Source>,
    name: &str,
    srgb: bool,
) -> Result<TextureBuilder<'static>, Error> {
    let (data, format) = get_image_data(&texture.source(), buffers, source, name.as_ref())?;

    let metadata = ImageTextureConfig {
        repr: match srgb {
            true => Repr::Srgb,
            false => Repr::Unorm,
        },
        format: match format {
            ImportDataFormat::Png => Some(DataFormat::PNG),
            ImportDataFormat::Jpeg => Some(DataFormat::JPEG),
        },
        sampler_info: load_sampler_info(&texture.sampler()),
        ..Default::default()
    };

    load_from_image(&data, metadata).map_err(|e| e.compat().into())
}

fn load_sampler_info(sampler: &gltf::texture::Sampler<'_>) -> hal::image::SamplerInfo {
    use gltf::texture::{MagFilter, MinFilter};
    use hal::image::{Filter, SamplerInfo};

    let mag_filter = match sampler.mag_filter() {
        None | Some(MagFilter::Nearest) => Filter::Nearest,
        Some(MagFilter::Linear) => Filter::Linear,
    };

    let (min_filter, mip_filter) = match sampler.min_filter() {
        None | Some(MinFilter::Nearest) | Some(MinFilter::NearestMipmapNearest) => {
            (Filter::Nearest, Filter::Nearest)
        }
        Some(MinFilter::Linear) | Some(MinFilter::LinearMipmapLinear) => {
            (Filter::Linear, Filter::Linear)
        }
        Some(MinFilter::NearestMipmapLinear) => (Filter::Nearest, Filter::Linear),
        Some(MinFilter::LinearMipmapNearest) => (Filter::Linear, Filter::Nearest),
    };

    let wrap_s = map_wrapping(sampler.wrap_s());
    let wrap_t = map_wrapping(sampler.wrap_t());

    let mut s = SamplerInfo::new(min_filter, wrap_s);
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
