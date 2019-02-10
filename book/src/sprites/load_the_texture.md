# Load The `Texture`

The first part of loading sprites into Amethyst is to read the image into memory. Currently Amethyst supports [`PngFormat`][doc_fmt_png], [`BmpFormat`][doc_fmt_bmp], [`JpgFormat`][doc_fmt_jpg] and [`TgaFormat`][doc_fmt_tga].

The following snippet shows how to load a PNG image:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{PngFormat, Texture, TextureMetadata, TextureHandle};

pub fn load_texture<N>(name: N, world: &World) -> TextureHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        PngFormat,
        TextureMetadata::srgb(),
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

* You don't get back the [`Texture`][doc_tex], but a [`TextureHandle`][doc_tex_hd], which is a cloneable reference to the texture.

    When you use [`loader.load(..)`][doc_load] to load an [`Asset`][doc_asset], the method returns immediately with a unique handle for your texture. The actual asset loading is handled asynchronously, so if you attempt to use the texture handle to retrieve the texture, such as with [`world.read_resource::<AssetStorage<Texture>>()`][doc_read_resource][`.get(texture_handle)`][doc_asset_get], you will get a `None` until the `Texture` has finished loading.

The loaded texture will use linear filter, e.g. screen pixels will be linearly interpolated between the closest image pixels. In layman's terms, if your images have small resolution, sprites will look blury. Use `TextureMetadata::srgb_scale()` instead to avoid such effect. Screen pixel will be taken from nearest pixel of texture in that case.

[doc_asset]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.AssetStorage.html#method.get
[doc_fmt_bmp]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.BmpFormat.html
[doc_fmt_jpg]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.JpgFormat.html
[doc_fmt_png]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.PngFormat.html
[doc_fmt_tga]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.TgaFormat.html
[doc_load]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.Loader.html#method.load
[doc_read_resource]: https://www.amethyst.rs/doc/latest/doc/specs/world/struct.World.html#method.read_resource
[doc_ss]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.SpriteSheet.html
[doc_tex]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Texture.html
[doc_tex_hd]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/type.TextureHandle.html
