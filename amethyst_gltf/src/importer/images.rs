use fnv::FnvHashMap;
use gltf::image::{Format, Data};
use image::buffer::ConvertBuffer;
use serde::{Deserialize, Serialize};
use crate::importer::GltfObjectId;
use gltf::image::Data as GltfImageData;
use image::RgbaImage;

use type_uuid::TypeUuid;
use amethyst_assets::resource_arc::ResourceArc;
use std::sync::Arc;
/*
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ImageAssetColorSpace {
    Srgb,
    Linear,
}

pub struct ImageToImport {
    pub id: GltfObjectId,
    pub asset: ImageAssetData,
}

#[derive(TypeUuid, Clone)]
#[uuid = "7a67b850-17f9-4877-8a6e-293a1589bbd8"]
pub struct ImageAsset {
    pub image: ResourceArc<ImageResource>,
    pub image_view: ResourceArc<ImageViewResource>,
}

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "e6166902-8716-401b-9d2e-8b01701c5626"]
pub struct ImageAssetData {
    pub width: u32,
    pub height: u32,
    pub color_space: ImageAssetColorSpace,

    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageKey {
    id: u64,
}

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub image: Arc<RafxImage>,
    // Dynamic resources have no key
    pub image_key: Option<ImageKey>,
}

/// Enum that holds either a texture or render target.
///
/// `texture()` and `texture_def()` can always be called, but `render_target()` may return none if
/// the image is a texture but not a render target
#[derive(Debug)]
pub enum RafxImage {
    Texture(RafxTexture),
    RenderTarget(RafxRenderTarget),
}

impl RafxImage {
    pub fn texture_def(&self) -> &RafxTextureDef {
        match self {
            RafxImage::Texture(inner) => inner.texture_def(),
            RafxImage::RenderTarget(inner) => inner.texture().texture_def(),
        }
    }

    pub fn texture(&self) -> &RafxTexture {
        match self {
            RafxImage::Texture(inner) => inner,
            RafxImage::RenderTarget(inner) => inner.texture(),
        }
    }

    pub fn render_target(&self) -> Option<&RafxRenderTarget> {
        match self {
            RafxImage::Texture(_inner) => None,
            RafxImage::RenderTarget(inner) => Some(inner),
        }
    }
}

impl From<RafxTexture> for RafxImage {
    fn from(texture: RafxTexture) -> Self {
        RafxImage::Texture(texture)
    }
}

impl From<RafxRenderTarget> for RafxImage {
    fn from(render_target: RafxRenderTarget) -> Self {
        RafxImage::RenderTarget(render_target)
    }
}

#[derive(Debug, Clone)]
pub enum RafxTextureBindType {
    Srv,
    SrvStencil,
    UavMipChain,
    UavMipSlice(u32),
}

#[derive(Debug, Clone)]
pub struct ImageViewResource {
    pub image: ResourceArc<ImageResource>,
    // Dynamic resources have no key
    pub image_view_key: Option<ImageViewKey>,
    pub texture_bind_type: Option<RafxTextureBindType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageViewKey {
    image_key: ImageKey,
    texture_bind_type: Option<RafxTextureBindType>,
}

impl std::fmt::Debug for ImageAssetData {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("width", &self.width)
            .field("width", &self.height)
            .field("byte_count", &self.data.len())
            .finish()
    }
}

pub fn build_image_color_space_assignments_from_materials(
    doc: &gltf::Document
) -> FnvHashMap<usize, ImageAssetColorSpace> {
    let mut image_color_space_assignments = FnvHashMap::default();

    for material in doc.materials() {
        let pbr_metallic_roughness = &material.pbr_metallic_roughness();

        if let Some(texture) = material.emissive_texture() {
            image_color_space_assignments.insert(
                texture.texture().source().index(),
                ImageAssetColorSpace::Srgb,
            );
        }else if let Some(texture) = material.occlusion_texture() {
            image_color_space_assignments.insert(
                texture.texture().source().index(),
                ImageAssetColorSpace::Srgb,
            );
        }else if let Some(texture) = material.normal_texture() {
            image_color_space_assignments.insert(
                texture.texture().source().index(),
                ImageAssetColorSpace::Linear,
            );
        } else if let Some(texture) = pbr_metallic_roughness.metallic_roughness_texture() {
            image_color_space_assignments.insert(
                texture.texture().source().index(),
                ImageAssetColorSpace::Linear,
            );
        } else if let Some(texture) = pbr_metallic_roughness.base_color_texture() {
            image_color_space_assignments.insert(
                texture.texture().source().index(),
                ImageAssetColorSpace::Srgb,
            );
        }
    }
    image_color_space_assignments
}

pub fn extract_images_to_import(
    doc: &gltf::Document,
    images: &[GltfImageData],
    image_color_space_assignments: &FnvHashMap<usize, ImageAssetColorSpace>,
) -> Vec<ImageToImport> {
    let mut images_to_import = Vec::with_capacity(images.len());
    for image in doc.images() {
        let image_data = &images[image.index()];
        let converted_image: image::RgbaImage = convert_image_format_to_rgba(&image_data);

        let color_space = *image_color_space_assignments
            .get(&image.index())
            .unwrap_or(&ImageAssetColorSpace::Linear);
        log::info!(
            "Choosing color space {:?} for image index {}",
            color_space,
            image.index()
        );

        let asset = ImageAssetData {
            data: converted_image.to_vec(),
            width: image_data.width,
            height: image_data.height,
            color_space,
        };
        let id = image
            .name()
            .map(|s| GltfObjectId::Name(s.to_string()))
            .unwrap_or_else(|| GltfObjectId::Index(image.index()));

        let image_to_import = ImageToImport { id, asset };

        // Verify that we iterate images in order so that our resulting assets are in order
        assert!(image.index() == images_to_import.len());
        log::debug!(
            "Importing Texture name: {:?} index: {} width: {} height: {} bytes: {}",
            image.name(),
            image.index(),
            image_to_import.asset.width,
            image_to_import.asset.height,
            image_to_import.asset.data.len()
        );

        images_to_import.push(image_to_import);
    }

    images_to_import
}

fn convert_image_format_to_rgba(image_data: &Data) -> RgbaImage {
    match image_data.format {
        Format::R8 => image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::R8G8 => image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::R8G8B8 => image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::R8G8B8A8 => image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::B8G8R8 => image::ImageBuffer::<image::Bgr<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::B8G8R8A8 => image::ImageBuffer::<image::Bgra<u8>, Vec<u8>>::from_vec(
            image_data.width,
            image_data.height,
            image_data.pixels.clone(),
        )
            .unwrap()
            .convert(),
        Format::R16 => {
            unimplemented!();
        }
        Format::R16G16 => {
            unimplemented!();
        }
        Format::R16G16B16 => {
            unimplemented!();
        }
        Format::R16G16B16A16 => {
            unimplemented!();
        }
    }
}
 */