use std::borrow::Cow;

use hal::{Backend, Device};
use hal::command::Offset;
use hal::device::Extent;
use hal::format::{AspectFlags, Format, Swizzle};
use hal::image::{ImageLayout, Kind, Level, SamplerInfo, SubresourceLayers, SubresourceRange, Usage};
use hal::memory::Properties;
use hal::pso::DescriptorWrite;
use hal::range::RangeArg;

use {Image, Error};
use factory::{Factory, ImageUpload};

/// Read-only image
pub struct Texture<B: Backend> {
    sampler: Option<B::Sampler>,
    image: Image<B>,
    view: B::ImageView,
    kind: Kind,
    levels: Level,
    format: Format,
}

impl<B> Texture<B>
where
    B: Backend,
{
    pub fn new<'a>() -> TextureBuilder<'a> {
        TextureBuilder::new()
    }

    /// Get bound sampler
    pub fn sampler(&self) -> Option<&B::Sampler> {
        self.sampler.as_ref()
    }

    /// Get raw image
    pub fn image(&self) -> &B::Image {
        self.image.raw()
    }

    /// Get view to image
    pub fn view(&self) -> &B::ImageView {
        &self.view
    }

    /// Get format of the texture
    pub fn format(&self) -> Format {
        self.format
    }

    /// Get kind of the texture
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Get number of levels of the texture
    pub fn levels(&self) -> Level {
        self.levels
    }
}

pub struct TextureBuilder<'a> {
    sampler: Option<SamplerInfo>,
    kind: Kind,
    levels: Level,
    format: Format,
    data: Option<Cow<'a, [u8]>>,
}

impl<'a> TextureBuilder<'a> {
    pub fn new() -> Self {
        TextureBuilder {
            sampler: None,
            kind: Kind::D1(1),
            levels: 1,
            format: Format::Rgba8Unorm,
            data: None,
        }
    }

    pub fn with_sampler(mut self, sampler: SamplerInfo) -> Self {
        self.sampler = Some(sampler);
        self
    }

    pub fn set_sampler(&mut self, sampler: SamplerInfo) -> &mut Self {
        self.sampler = Some(sampler);
        self
    }

    pub fn with_kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    pub fn set_kind(&mut self, kind: Kind) -> &mut Self {
        self.kind = kind;
        self
    }

    pub fn set_raw_data<D>(&mut self, data: D) -> &mut Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.data = Some(data.into());
        self
    }

    pub fn with_raw_data<D>(mut self, data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.set_raw_data(data);
        self
    }

    pub fn set_format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }

    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn build<B>(&self, factory: &mut Factory<B>) -> Result<Texture<B>, Error>
    where
        B: Backend,
    {
        let mut image = factory.create_image(Properties::DEVICE_LOCAL, self.kind, self.levels, self.format, Usage::SAMPLED | Usage::TRANSFER_DST)?;

        if let Some(ref data) = self.data {
            // Check that data provided matches bits of the whole image.
            let desc = self.format.base_format().0.desc();
            let (width, height, depth, _) = self.kind.dimensions();
            let width = if width == 0 { 0usize } else { (width as usize - 1) / desc.dim.0 as usize + 1 };
            let height = if height == 0 { 0usize } else { (height as usize - 1) / desc.dim.1 as usize + 1 };
            let depth = depth as usize;
            let blocks = width * height * depth;
            let total_bits = blocks * desc.bits as usize;
            assert_eq!(total_bits, data.len() * 8);

            factory.upload_image(&mut image, data, ImageLayout::General, ImageUpload {
                layers: SubresourceLayers {
                    aspects: self.format.aspect_flags(),
                    level: 0,
                    layers: 0 .. 1,
                },
                offset: Offset {
                    x: 0,
                    y: 0,
                    z: 0,
                },
                extent: {
                    let (x, y, z, _) = self.kind.dimensions();
                    Extent {
                        width: x as u32,
                        height: y as u32,
                        depth: z as u32,
                    }
                }
            })?;
        }

        let view = factory.create_image_view(&image.raw(), self.format, Swizzle::NO, SubresourceRange {
            aspects: self.format.aspect_flags(),
            levels: 0 .. self.kind.num_levels(),
            layers: 0 .. self.kind.num_layers(),
        }).unwrap();

        Ok(Texture {
            sampler: self.sampler.clone().map(|info| factory.create_sampler(info)),
            image,
            view,
            kind: self.kind,
            levels: self.levels,
            format: self.format,
        })
    }
}
