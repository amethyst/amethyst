//! Texture resource.

use std::marker::PhantomData;

use gfx_hal::Backend;
use gfx_hal::image::{AaMode, AspectFlags, FilterMethod, Kind, Level, SamplerInfo,
                     SubresourceRange, Usage, WrapMode};
use gfx_hal::format::{Format, ImageFormat, Rgba8, SurfaceTyped, Swizzle};
use gfx_hal::memory::Pod;

const COLOR_RANGE: SubresourceRange = SubresourceRange {
    aspects: AspectFlags::COLOR,
    levels: 0..1,
    layers: 0..1,
};

error_chain!{}

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
pub struct TextureBuilder<D, T> {
    image: ImageInfo,
    view: ViewInfo,
    sampler: SamplerInfo,
    data: D,
    pd: PhantomData<T>,
}

impl TextureBuilder<[[u8; 4]; 1], Rgba8> {
    /// Creates a new `TextureBuilder` from the given RGBA color value.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> Self {
        let rgba = rgba.into();
        let r = rgba[0];
        let g = rgba[1];
        let b = rgba[2];
        let a = rgba[3];
        TextureBuilder::new([
            [
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            ],
        ])
    }
}

impl<D, F, S, P> TextureBuilder<D, F>
where
    F: ImageFormat<Surface = S>,
    S: SurfaceTyped<DataType = P>,
    P: Pod,
    D: AsRef<[P]>,
{
    /// Creates a new `TextureBuilder` with the given raw texture data.
    pub fn new(data: D) -> Self {
        TextureBuilder {
            image: ImageInfo {
                kind: Kind::D1(data.as_ref().len() as u16),
                levels: 1,
                usage: Usage::SAMPLED,
            },
            view: ViewInfo {
                swizzle: Swizzle::NO,
                range: COLOR_RANGE,
            },
            sampler: SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp),
            data: data,
            pd: PhantomData,
        }
    }

    /// Sets the `SamplerInfo` for the texture
    pub fn with_sampler(mut self, sampler: SamplerInfo) -> Self {
        self.sampler = sampler;
        self
    }

    /// Sets the number of mipmap levels to generate.
    ///
    /// FIXME: Only encoders can generate mipmap levels.
    pub fn mip_levels(mut self, val: u8) -> Self {
        self.image.levels = val;
        self
    }

    /// Builds and returns the new texture.
    pub fn build<B>(self) -> Result<Texture<B>>
    where
        B: Backend,
    {
        unimplemented!()
    }
}


/// Handle to a GPU texture resource.
#[derive(Debug)]
pub struct Texture<B: Backend> {
    sampler: B::Sampler,
    view: B::ImageView,
    image: B::Image,
}

impl<B> Texture<B>
where
    B: Backend,
{
    /// Builds a new texture with the given raw texture data.
    pub fn from_data<D, F, S, P>(data: D) -> TextureBuilder<D, F>
    where
        F: ImageFormat<Surface = S>,
        S: SurfaceTyped<DataType = P>,
        P: Pod,
        D: AsRef<[P]>,
    {
        TextureBuilder::new(data)
    }

    /// Builds a new texture with the given raw texture data.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> TextureBuilder<[[u8; 4]; 1], Rgba8> {
        TextureBuilder::from_color_val(rgba)
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
