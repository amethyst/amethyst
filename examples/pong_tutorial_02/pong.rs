use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        cgmath::{Matrix4, Vector3},
        transform::{GlobalTransform, Transform},
    },
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::{
        Camera, MaterialTextureSet, PngFormat, Projection, Sprite, SpriteRender, SpriteSheet,
        SpriteSheetHandle, Texture, TextureCoordinates, TextureMetadata,
    },
};

const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;

const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_WIDTH: f32 = 4.0;

const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);

pub struct Pong;

impl<'a, 'b> SimpleState<'a, 'b> for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
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
enum Side {
    Left,
    Right,
}

struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side: side,
            width: 1.0,
            height: 1.0,
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
    // `texture_id` is a application defined ID given to the texture to store in the `World`.
    // This is needed to link the texture to the sprite_sheet.
    let texture_id = 0;
    let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
    material_texture_set.insert(texture_id, texture_handle);

    // Create the sprite for the paddles.
    //
    // Texture coordinates are expressed as a proportion of the sprite sheet's dimensions between
    // 0.0 and 1.0, so they must be divided by the width or height.
    //
    // In addition, on the Y axis, texture coordinates are 0.0 at the bottom of the sprite sheet and
    // 1.0 at the top, which is the opposite direction of pixel coordinates, so we have to invert
    // the value by subtracting the pixel proportion from 1.0.
    let tex_coords = TextureCoordinates {
        left: 0.0,
        right: PADDLE_WIDTH / SPRITESHEET_SIZE.0,
        bottom: 1.0 - PADDLE_HEIGHT / SPRITESHEET_SIZE.1,
        top: 1.0,
    };
    let paddle_sprite = Sprite {
        width: PADDLE_WIDTH,
        height: PADDLE_HEIGHT,
        offsets: [PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0],
        tex_coords,
    };

    // Collate the sprite layout information into a sprite sheet
    let sprite_sheet = SpriteSheet {
        texture_id,
        sprites: vec![paddle_sprite],
    };

    let sprite_sheet_handle = {
        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load_from_data(sprite_sheet, (), &sprite_sheet_store)
    };

    sprite_sheet_handle
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            ARENA_HEIGHT,
            0.0,
        ))).with(GlobalTransform(
            Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into(),
        )).build();
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.translation = Vector3::new(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.translation = Vector3::new(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Assign the sprites for the paddles
    let sprite_render_left = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
        flip_horizontal: false,
        flip_vertical: false,
    };

    let sprite_render_right = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
        flip_horizontal: true,
        flip_vertical: false,
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with(sprite_render_left)
        .with(Paddle::new(Side::Left))
        .with(GlobalTransform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(sprite_render_right)
        .with(Paddle::new(Side::Right))
        .with(GlobalTransform::default())
        .with(right_transform)
        .build();
}
