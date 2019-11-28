# Load The `Texture`

The first part of loading sprites into Amethyst is to read the image into memory.

The following snippet shows how to load a PNG / JPEG / GIF / ICO image:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{formats::texture::ImageFormat, Texture};

pub fn load_texture<N>(name: N, world: &World) -> Handle<Texture>
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
    }
}
#
# fn main() {}
```

There is one thing that may surprise you.

* You don't get back the [`Texture`][doc_tex], but a [`Handle<Texture>`][doc_tex_hd], which is a 
cloneable reference to the texture.

    When you use [`loader.load(..)`][doc_load] to load an [`Asset`][doc_asset], the method returns immediately with a unique handle for your texture. The actual asset loading is handled asynchronously, so if you attempt to use the texture handle to retrieve the texture, such as with [`world.read_resource::<AssetStorage<Texture>>()`][doc_read_resource][`.get(texture_handle)`][doc_asset_get], you will get a `None` until the `Texture` has finished loading.

The loaded texture will use nearest filtering, i.e. the pixels won't be interpolated.
If you want to tweak the sampling, you can change `ImageFormat::default()` to
`ImageFormat(my_config)`, and create your own `my_config` like this:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::renderer::rendy::hal::image::{Filter, SamplerInfo, WrapMode};
use amethyst::renderer::rendy::texture::image::{ImageTextureConfig, Repr, TextureKind};

let my_config = ImageTextureConfig {
    // Determine format automatically
    format: None,
    // Color channel
    repr: Repr::Srgb,
    // Two-dimensional texture
    kind: TextureKind::D2,
    sampler_info: SamplerInfo::new(Filter::Linear, WrapMode::Clamp),
    // Don't generate mipmaps for this image
    generate_mips: false,
    premultiply_alpha: true,
};
```

[doc_asset]: https://docs.amethyst.rs/stable/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://docs.amethyst.rs/stable/amethyst_assets/struct.AssetStorage.html#method.get
[doc_fmt_bmp]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.BmpFormat.html
[doc_fmt_jpg]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.JpgFormat.html
[doc_fmt_png]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.PngFormat.html
[doc_fmt_tga]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.TgaFormat.html
[doc_load]: https://docs.amethyst.rs/stable/amethyst_assets/struct.Loader.html#method.load
[doc_read_resource]: https://docs.amethyst.rs/stable/specs/world/struct.World.html#method.read_resource
[doc_ss]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.SpriteSheet.html
[doc_tex]: https://docs.amethyst.rs/stable/amethyst_renderer/struct.Texture.html
[doc_tex_hd]: https://docs.amethyst.rs/stable/amethyst_assets/type.Handle.html
