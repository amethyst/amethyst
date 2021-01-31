# Load The `Texture`

The first part of loading sprites into Amethyst is to read the image into memory.

The following snippet shows how to load a PNG / JPEG / GIF / ICO image:

```rust
use amethyst::assets::{AssetStorage, DefaultLoader, Handle, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{formats::texture::ImageFormat, Texture};

pub fn load_texture<N>(name: N, world: &World) -> Handle<Texture>
where
    N: Into<String>,
{
    let loader = resources.get::<DefaultLoader>();
    loader.load(
        name,
    )
}

#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
    }
}
# fn main() {}
```

There is one thing that may surprise you.

- You don't get back the [`Texture`][doc_tex], but a [`Handle<Texture>`][doc_tex_hd], which is a
  cloneable reference to the texture.

  When you use [`loader.load(..)`][doc_load] to load an [`Asset`][doc_asset], the method returns immediately with a unique handle for your texture. The actual asset loading is handled asynchronously, so if you attempt to use the texture handle to retrieve the texture, such as with [`resources.get::<AssetStorage<Texture>>()`][doc_read_resource][`.get(texture_handle)`][doc_asset_get], you will get a `None` until the `Texture` has finished loading.

The loaded texture will use nearest filtering, i.e. the pixels won't be interpolated.
If you want to tweak the sampling, you can change `ImageFormat::default()` to
`ImageFormat(my_config)`, and create your own `my_config` like this:

```rust
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

[doc_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://docs.amethyst.rs/master/amethyst_assets/struct.AssetStorage.html#method.get
[doc_load]: https://docs.amethyst.rs/master/amethyst_assets/struct.Loader.html#method.load
[doc_read_resource]: https://docs.rs/specs/~0.16/specs/world/struct.World.html#method.read_resource
[doc_tex]: https://docs.amethyst.rs/master/amethyst_rendy/rendy/texture/struct.Texture.html
[doc_tex_hd]: https://docs.amethyst.rs/master/amethyst_assets/struct.Handle.html
