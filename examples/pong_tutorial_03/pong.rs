use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::{
        Camera, Flipped, PngFormat, Projection, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata,
    },
};

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub struct Pong;

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        let sprite_sheet_handle = load_sprite_sheet(world);

        world.register::<Paddle>();

        initialise_paddles(world, sprite_sheet_handle);
        initialise_camera(world);
    }
}

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side: side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `sprite_sheet` is the layout of the sprites on the image
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/pong_spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/pong_spritesheet.ron", // Here we load the associated ron file
        SpriteSheetFormat,
        texture_handle, // We pass it the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::<f32>::default();
    transform.set_translation_xyz(0.0, 0.0, 1.0);

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            0.0,
            ARENA_HEIGHT,
        )))
        .with(transform)
        .build();
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    let mut left_transform = Transform::<f32>::default();
    let mut right_transform = Transform::<f32>::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Assign the sprites for the paddles
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Flipped::Horizontal)
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .build();
}
