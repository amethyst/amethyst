use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::World;
use amethyst::renderer::{
    FilterMethod, MaterialTextureSet, PngFormat, SamplerInfo, SpriteSheet, SpriteSheetFormat,
    SpriteSheetHandle, Texture, TextureMetadata, WrapMode,
};

const TEXTURE_ID: u64 = 0;

pub fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/isometric_tiles.png",
            PngFormat,
            TextureMetadata::default()
                .with_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Border)),
            (),
            &texture_storage,
        )
    };

    let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
    material_texture_set.insert(TEXTURE_ID, texture_handle);

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/isometric_tiles.ron",
        SpriteSheetFormat,
        TEXTURE_ID,
        (),
        &sprite_sheet_store,
    )
}
