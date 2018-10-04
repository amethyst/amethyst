use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        cgmath::{Matrix4, Vector3},
        transform::{GlobalTransform, Transform},
    },
    ecs::prelude::World,
    prelude::*,
    renderer::{
        Camera, MaterialTextureSet, PngFormat, Projection, Sprite, SpriteRender, SpriteSheet,
        SpriteSheetHandle, Texture, TextureCoordinates, WindowMessages,
    },
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use systems::ScoreText;
use {Ball, Paddle, Side};
use {ARENA_HEIGHT, ARENA_WIDTH, SPRITESHEET_SIZE};

pub struct Pong;

impl<'a, 'b> SimpleState<'a, 'b> for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        use audio::initialise_audio;

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        let sprite_sheet_handle = load_sprite_sheet(world);

        // Setup our game.
        initialise_paddles(world, sprite_sheet_handle.clone());
        initialise_ball(world, sprite_sheet_handle);
        initialise_camera(world);
        initialise_audio(world);
        initialise_score(world);
        hide_cursor(world);
    }
}

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    use {BALL_RADIUS, PADDLE_HEIGHT, PADDLE_WIDTH};

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
            Default::default(),
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

    // Create the sprite for the ball.
    let ball_diameter = BALL_RADIUS * 2.0;
    let tex_coords = TextureCoordinates {
        left: PADDLE_WIDTH / SPRITESHEET_SIZE.0,
        right: 1.0,
        bottom: 1.0 - ball_diameter / SPRITESHEET_SIZE.1,
        top: 1.0,
    };
    let ball_sprite = Sprite {
        width: ball_diameter,
        height: ball_diameter,
        offsets: [BALL_RADIUS, BALL_RADIUS],
        tex_coords,
    };

    // Collate the sprite layout information into a sprite sheet
    let sprite_sheet = SpriteSheet {
        texture_id,
        sprites: vec![paddle_sprite, ball_sprite],
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

/// Hide the cursor, so it's invisible while playing.
fn hide_cursor(world: &mut World) {
    world
        .write_resource::<WindowMessages>()
        .send_command(|win| win.hide_cursor(true));
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    use {PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = (ARENA_HEIGHT - PADDLE_HEIGHT) / 2.0;
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
        .with(Paddle {
            side: Side::Left,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        }).with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(sprite_render_right)
        .with(Paddle {
            side: Side::Right,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        }).with(right_transform)
        .build();
}

/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    use {ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y};

    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.translation = Vector3::new(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite_sheet
        flip_horizontal: true,
        flip_vertical: false,
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        }).with(local_transform)
        .build();
}

fn initialise_score(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        Default::default(),
        (),
        &world.read_resource(),
    );
    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        -50.,
        50.,
        1.,
        55.,
        50.,
        0,
    );

    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        50.,
        50.,
        1.,
        55.,
        50.,
        0,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        )).build();
    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        )).build();
    world.add_resource(ScoreText { p1_score, p2_score });
}
