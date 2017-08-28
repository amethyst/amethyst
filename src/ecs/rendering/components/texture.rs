//! Texture resource handling.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use futures::{Async, Future, Poll};
use gfx::format::SurfaceType;
use imagefmt::ColFmt;
use rayon::ThreadPool;



use assets::{Asset, AssetFuture, AssetPtr, AssetSpec, Cache, Context};
use assets::formats::textures::ImageData;
use ecs::{Component, VecStorage};
use ecs::rendering::resources::{Factory, FactoryFuture};
use renderer::{Texture, TextureBuilder, Error as RendererError};



/// Error that can occur during texture creation
#[derive(Debug)]
pub enum TextureError {
    /// Error occured in renderer
    Renderer(RendererError),

    /// Color format unsupported
    UnsupportedColorFormat(ColFmt),

    /// Texture is oversized
    UnsupportedSize {
        /// Maximum size of texture (width, height)
        max: (usize, usize),

        /// Image size (width, height)
        got: (usize, usize),
    },
}

impl Error for TextureError {
    fn description(&self) -> &str {
        match *self {
            TextureError::Renderer(ref err) => err.description(),
            TextureError::UnsupportedColorFormat(_) => "Unsupported color format",
            TextureError::UnsupportedSize { .. } => "Unsupported size",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TextureError::Renderer(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for TextureError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            TextureError::Renderer(ref err) => write!(fmt, "Render error: {}", err),
            TextureError::UnsupportedColorFormat(col_fmt) => {
                write!(fmt, "Unsupported color format: {:?}", col_fmt)
            }
            TextureError::UnsupportedSize { max, got } => {
                write!(fmt, "Unsupported size. max: {:?}, got: {:?}", max, got)
            }
        }
    }
}


enum Inner {
    Factory(FactoryFuture<Texture, RendererError>),
    Err(Option<TextureError>),
}

/// Will be `TextureComponent` result type of `TextureContext::create_asset`
pub struct TextureFuture(Inner);

impl TextureFuture {
    fn factory(future: FactoryFuture<Texture, RendererError>) -> Self {
        TextureFuture(Inner::Factory(future))
    }

    fn from_error(error: TextureError) -> Self {
        TextureFuture(Inner::Err(Some(error)))
    }

    fn unsupported_color_format(fmt: ColFmt) -> Self {
        Self::from_error(TextureError::UnsupportedColorFormat(fmt))
    }

    fn unsupported_size(width: usize, height: usize) -> Self {
        Self::from_error(TextureError::UnsupportedSize {
                             max: (u16::max_value() as usize, u16::max_value() as usize),
                             got: (width, height),
                         })
    }
}

impl Future for TextureFuture {
    type Item = TextureComponent;
    type Error = TextureError;

    fn poll(&mut self) -> Poll<TextureComponent, TextureError> {
        match self.0 {
            Inner::Factory(ref mut future) => {
                match future.poll() {
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Ok(Async::Ready(texture)) => Ok(Async::Ready(TextureComponent::new(texture))),
                    Err(err) => Err(TextureError::Renderer(err)),
                }
            }
            Inner::Err(ref mut err) => Err(err.take().expect("polling completed future")),
        }
    }
}



/// Wraps `Texture` into component
#[derive(Clone, Debug)]
pub struct TextureComponent(pub AssetPtr<Texture, TextureComponent>);

impl AsRef<Texture> for TextureComponent {
    fn as_ref(&self) -> &Texture {
        self.0.inner_ref()
    }
}

impl AsMut<Texture> for TextureComponent {
    fn as_mut(&mut self) -> &mut Texture {
        self.0.inner_mut()
    }
}

impl TextureComponent {
    /// Create new `TextureComponent` from `Texture`
    pub fn new(texture: Texture) -> Self {
        TextureComponent(AssetPtr::new(texture))
    }
}

impl Component for TextureComponent {
    type Storage = VecStorage<Self>;
}

impl Asset for TextureComponent {
    type Context = TextureContext;
}

/// Context to create textures from images
pub struct TextureContext {
    cache: Cache<AssetFuture<TextureComponent>>,
    factory: Factory,
}

impl TextureContext {
    pub(crate) fn new(factory: Factory) -> Self {
        TextureContext {
            cache: Cache::new(),
            factory: factory,
        }
    }
}

impl Context for TextureContext {
    type Asset = TextureComponent;
    type Data = ImageData;
    type Error = TextureError;
    type Result = TextureFuture;

    fn category(&self) -> &'static str {
        "texture"
    }

    fn create_asset(&self, image: ImageData, _: &ThreadPool) -> TextureFuture {
        fn convert_color_format(fmt: ColFmt) -> Option<SurfaceType> {
            match fmt {
                ColFmt::Auto => unreachable!(),
                ColFmt::RGBA => Some(SurfaceType::R8_G8_B8_A8),
                ColFmt::BGRA => Some(SurfaceType::B8_G8_R8_A8),
                _ => None,
            }
        }

        let image = image.raw;
        let fmt = match convert_color_format(image.fmt) {
            Some(fmt) => fmt,
            None => return TextureFuture::unsupported_color_format(image.fmt),
        };

        if image.w > u16::max_value() as usize || image.h > u16::max_value() as usize {
            return TextureFuture::unsupported_size(image.w, image.h);
        }

        let tb = TextureBuilder::new(image.buf)
            .with_format(fmt)
            .with_size(image.w as u16, image.h as u16);
        TextureFuture::factory(self.factory.create_texture(tb))
    }

    fn cache(&self, spec: AssetSpec, asset: AssetFuture<TextureComponent>) {
        self.cache.insert(spec, asset);
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetFuture<TextureComponent>> {
        self.cache.get(spec)
    }

    fn update(&self, spec: &AssetSpec, asset: AssetFuture<TextureComponent>) {
        if let Some(asset) = self.cache
               .access(spec, |a| match a.peek() {
            Some(Ok(a)) => {
                a.0.push_update(asset);
                None
            }
            _ => Some(asset),
        }).and_then(|a| a) {
            self.cache.insert(spec.clone(), asset);
        }
    }

    fn clear(&self) {
        self.cache.retain(|_, a| match a.peek() {
            Some(Ok(a)) => a.0.is_shared(),
            _ => true,
        });
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
