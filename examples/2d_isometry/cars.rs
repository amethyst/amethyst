use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::{IsometricTransform, Time};
use amethyst::ecs::{Component, Join, Read, System, VecStorage, World, WriteStorage};
use amethyst::prelude::*;
use amethyst::renderer::{
    FilterMethod, MaterialTextureSet, PngFormat, SamplerInfo, Sprite, SpriteRender, SpriteSheet,
    SpriteSheetHandle, Texture, TextureCoordinates, TextureMetadata, Transparent, WrapMode,
};

use map::UNIT_DIMENSIONS;

const CARS_TEXTURE_ID: u64 = 1;

pub struct Car {
    top: f32,
    bottom: f32,
    speed: f32,
    up_sprite: usize,
    down_sprite: usize,
}

impl Component for Car {
    type Storage = VecStorage<Self>;
}

pub fn initialise_cars(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let mut iso_transf =
        IsometricTransform::from_unit_dimensions(UNIT_DIMENSIONS.0, UNIT_DIMENSIONS.1);
    iso_transf.translation.x = 1.5;
    iso_transf.translation.y = 1.3;
    iso_transf.order_priority = 300.0;
    world
        .create_entity()
        .with(iso_transf)
        .with(Car {
            top: 2.3,
            bottom: 0.0,
            speed: 0.5,
            up_sprite: 0,
            down_sprite: 1,
        })
        .with(SpriteRender {
            sprite_sheet,
            sprite_number: 0,
            flip_horizontal: false,
            flip_vertical: false,
        })
        .with(Transparent)
        .build();
}

pub fn load_cars_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/vehicles_blue.png",
            PngFormat,
            TextureMetadata::default()
                .with_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Border)),
            (),
            &texture_storage,
        )
    };

    let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
    material_texture_set.insert(CARS_TEXTURE_ID, texture_handle);

    let mut sprites = Vec::new();
    let tex_coords = TextureCoordinates {
        left: 1.0 / 256.0,
        right: 32.0 / 256.0,
        bottom: 1.0 - 278.0 / 512.0,
        top: 1.0 - 247.0 / 512.0,
    };
    sprites.push(Sprite {
        width: 32.0,
        height: 32.0,
        offsets: [0.0, 0.0],
        tex_coords,
    });

    let tex_coords = TextureCoordinates {
        left: 33.0 / 256.0,
        right: 64.0 / 256.0,
        bottom: 1.0 - 302.0 / 512.0,
        top: 1.0 - 272.0 / 512.0,
    };
    sprites.push(Sprite {
        width: 32.0,
        height: 31.0,
        offsets: [0.0, 0.0],
        tex_coords,
    });

    let sprite_sheet = SpriteSheet {
        texture_id: CARS_TEXTURE_ID,
        sprites: sprites,
    };

    let sprite_sheet_handle = {
        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load_from_data(sprite_sheet, (), &sprite_sheet_store)
    };

    sprite_sheet_handle
}

pub struct MoveCars;

impl<'a> System<'a> for MoveCars {
    type SystemData = (
        WriteStorage<'a, Car>,
        WriteStorage<'a, IsometricTransform>,
        WriteStorage<'a, SpriteRender>,
        Read<'a, Time>,
    );

    fn run(&mut self, (mut cars, mut transfs, mut sprites, time): Self::SystemData) {
        for (car, transf, sprite) in (&mut cars, &mut transfs, &mut sprites).join() {
            transf.translation.x += car.speed * time.delta_seconds();
            if car.speed > 0.0 {
                if transf.translation.x > car.top {
                    transf.translation.x = car.top;
                    sprite.sprite_number = car.down_sprite;
                    car.speed = -car.speed;
                }
            } else if car.speed < 0.0 {
                if transf.translation.x < car.bottom {
                    transf.translation.x = car.bottom;
                    sprite.sprite_number = car.up_sprite;
                    car.speed = -car.speed;
                }
            }
        }
    }
}
