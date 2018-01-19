//! Texture resource.

use std::marker::PhantomData;
use std::mem::size_of;

use assets::{Asset, AssetStorage, Handle};

use gfx_hal::{Backend, Device};
use gfx_hal::command::{BufferImageCopy, Offset};
use gfx_hal::device::Extent;
use gfx_hal::format::{AspectFlags, Format, Swizzle};
use gfx_hal::image::{FilterMethod, ImageLayout, Kind, Level, SamplerInfo, SubresourceLayers,
                     SubresourceRange, Usage, ViewError, WrapMode};
use gfx_hal::memory::{Pod, Properties};

use specs::DenseVecStorage;

use epoch::CurrentEpoch;
use formats::TextureData;
use memory::{cast_vec, Allocator, Image};
use relevant::Relevant;
use upload::Uploader;

const COLOR_RANGE: SubresourceRange = SubresourceRange {
    aspects: AspectFlags::COLOR,
    levels: 0..1,
    layers: 0..1,
};

const COLOR_LAYER: SubresourceLayers = SubresourceLayers {
    aspects: AspectFlags::COLOR,
    level: 0,
    layers: 0..1,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct ImageInfo {
    kind: Kind,
    levels: Level,
    usage: Usage,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct ViewInfo {
    swizzle: Swizzle,
    range: SubresourceRange,
}

/// Builds new textures.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TextureBuilder {
    format: Format,
    image: ImageInfo,
    row_pitch: Option<u32>,
    view: ViewInfo,
    sampler: SamplerInfo,
    data: Vec<u8>,
}

impl TextureBuilder {
    /// Creates a new `TextureBuilder` from the given RGBA color value.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> Self {
        let rgba = rgba.into();
        TextureBuilder::new(vec![rgba[0], rgba[1], rgba[2], rgba[3]])
    }
}

impl TextureBuilder {
    /// Creates a new `TextureBuilder` with the given raw texture data.
    pub fn new<D, P>(data: D) -> Self
    where
        D: AsRef<[P]> + Into<Vec<P>>,
        P: Pod,
    {
        let len = data.as_ref().len();
        TextureBuilder {
            format: Format::Rgba8Unorm,
            image: ImageInfo {
                kind: Kind::D1(len as u16 / 4),
                levels: 1,
                usage: Usage::SAMPLED,
            },
            row_pitch: None,
            view: ViewInfo {
                swizzle: Swizzle::NO,
                range: COLOR_RANGE,
            },
            sampler: SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp),
            data: cast_vec(data.into()),
        }
    }

    /// Builds and returns the new texture.
    pub fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<Texture<B>, ::failure::Error>
    where
        B: Backend,
    {
        let mut image = allocator.create_image(
            device,
            self.image.kind,
            self.image.levels,
            self.format,
            self.image.usage,
            Properties::DEVICE_LOCAL,
        )?;

        let bytes = self.data.len() as u32;
        let pixel_bits = self.format.base_format().0.desc().bits;
        assert_eq!(0, pixel_bits % 8);
        let pixel = pixel_bits / 8;

        let mut copy = BufferImageCopy {
            buffer_offset: 0,
            buffer_width: 0,
            buffer_height: 0,
            image_layers: COLOR_LAYER,
            image_offset: Offset { x: 0, y: 0, z: 0 },
            image_extent: Extent {
                width: 1,
                height: 1,
                depth: 1,
            },
        };

        match self.image.kind {
            Kind::D1(width) => {
                copy.buffer_width = self.row_pitch.unwrap_or(0);
                copy.image_extent.width = width.into();
            }
            Kind::D2(width, height, _) => {
                copy.buffer_width = self.row_pitch.unwrap_or(0);
                copy.image_extent.width = width.into();
                copy.image_extent.height = height.into();
            }
            _ => unimplemented!(),
        };

        uploader.upload_image(
            allocator,
            current,
            device,
            &mut image,
            ImageLayout::ShaderReadOnlyOptimal,
            copy,
            self.data,
        )?;

        let view =
            device.create_image_view(image.raw(), self.format, self.view.swizzle, self.view.range)?;

        let sampler = device.create_sampler(self.sampler);
        Ok(Texture {
            relevant: Relevant,
            sampler,
            view,
            image,
        })
    }

    /// Set texture format
    pub fn with_format(mut self, format: Format) -> Self {
        assert_eq!(format.base_format().0.desc().dim, (1, 1));
        assert_eq!(format.base_format().0.desc().aspects, AspectFlags::COLOR);
        self.format = format;
        self
    }

    /// Set data row pitch
    pub fn with_row_pitch(mut self, row_pitch: u32) -> Self {
        self.row_pitch = Some(row_pitch);
        self
    }

    /// Set kind of the texture
    pub fn with_kind(mut self, kind: Kind) -> Self {
        self.image.kind = kind;
        self
    }

    /// Sets the `SamplerInfo` for the texture
    pub fn with_sampler(mut self, sampler: SamplerInfo) -> Self {
        self.sampler = sampler;
        self
    }

    /// Sets the number of mipmap levels to generate.
    pub fn mip_levels(mut self, _val: u8) -> Self {
        unimplemented!()
    }
}

/// Handle to a GPU texture resource.
#[derive(Debug)]
pub struct Texture<B: Backend> {
    relevant: Relevant,
    sampler: B::Sampler,
    view: B::ImageView,
    image: Image<B>,
}

impl<B> Texture<B>
where
    B: Backend,
{
    /// Builds a new texture with the given raw texture data.
    pub fn from_data<D, P>(data: D) -> TextureBuilder
    where
        D: AsRef<[P]> + Into<Vec<P>>,
        P: Pod,
    {
        TextureBuilder::new(data)
    }

    /// Builds a new texture with the given raw texture data.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> TextureBuilder {
        TextureBuilder::from_color_val(rgba).with_format(Format::Rgba32Float)
    }

    /// Returns the sampler for the texture.
    pub fn sampler(&self) -> &B::Sampler {
        &self.sampler
    }

    /// Returns the texture's raw shader resource view.
    pub fn view(&self) -> &B::ImageView {
        &self.view
    }
}

/// A handle to a `Texture` asset.
pub type TextureHandle<B: Backend> = Handle<Texture<B>>;

/// A storage of the `Texture`'s.
pub type TextureStorage<B: Backend> = AssetStorage<Texture<B>>;

impl<B> Asset for Texture<B>
where
    B: Backend,
{
    const NAME: &'static str = "Texture";
    type Data = TextureData;
    type HandleStorage = DenseVecStorage<TextureHandle<B>>;
}
