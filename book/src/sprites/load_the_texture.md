# Load The `Texture`

The first part of loading sprites into `Amethyst` is to read the image into memory. Currently `Amethyst` supports [`PngFormat`][doc_fmt_png], [`BmpFormat`][doc_fmt_bmp], [`JpgFormat`][doc_fmt_jpg] and [`TgaFormat`][doc_fmt_tga].

The following snippet shows how to load a PNG image:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{MaterialTextureSet, PngFormat, Texture, TextureHandle};

pub fn load_texture<N>(name: N, world: &World) -> TextureHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        PngFormat,
        Default::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

#[derive(Debug)]
struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);

        let texture_id = 0;
        data.world
            .write_resource::<MaterialTextureSet>()
            .insert(texture_id, texture_handle);
    }
}
#
# fn main() {}
```

There are two things that may surprise you.

* Firstly, you don't get back the [`Texture`][doc_tex], but a [`TextureHandle`][doc_tex_hd], which is a cloneable reference to the texture.

    When you use [`loader.load(..)`][doc_load] to load an [`Asset`][doc_asset], the method returns immediately with a unique handle for your texture. The actual asset loading is handled asynchronously, so if you attempt to use the texture handle to retrieve the texture, such as with [`world.read_storage<Texture>()`][doc_read_storage][`.get(texture_handle)`][doc_asset_get], you may get a `None` for a few seconds.

* Secondly, you have to insert the texture into a `MaterialTextureSet`, with an arbitrary `u64` ID.

    The ID is necessary to link the [`Texture`][doc_tex] (loaded image) to the [`SpriteSheet`][doc_ss] (layout data), which takes the texture ID instead of the handle.

    You pick the texture ID based on how you want to reference it. For example, you might have an application configuration that says `path/to/spritesheet_0.png` is ID `100`, `path/to/spritesheet_1.png` is ID `101`, so you can use that. Or, you might do something clever like calculate an ID based on the path, and if it's already loaded, then you know you don't have to load it again.

[doc_asset]: https://docs.rs/amethyst_assets/latest/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://docs.rs/amethyst_assets/latest/amethyst_assets/struct.AssetStorage.html#method.get
[doc_fmt_bmp]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.BmpFormat.html
[doc_fmt_jpg]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.JpgFormat.html
[doc_fmt_png]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.PngFormat.html
[doc_fmt_tga]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.TgaFormat.html
[doc_load]: https://docs.rs/amethyst_assets/load/amethyst_assets/struct.Loader.html#method.load
[doc_read_storage]: https://docs.rs/specs/latest/specs/world/struct.World.html#method.read_storage
[doc_ss]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.SpriteSheet.html
[doc_tex]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/struct.Texture.html
[doc_tex_hd]: https://docs.rs/amethyst_renderer/latest/amethyst_renderer/type.TextureHandle.html
