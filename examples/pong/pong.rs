use amethyst::{
    assets::{AssetStorage, Loader},
    core::{cgmath::Vector3, transform::Transform},
    ecs::prelude::World,
    prelude::*,
    renderer::{
        Camera, MaterialTextureSet, PngFormat, Projection, SpriteRender, SpriteSheet,
        SpriteSheetFormat, SpriteSheetHandle, Texture, TextureMetadata,
    },
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use systems::ScoreText;
use {Ball, Paddle, Side};
use {ARENA_HEIGHT, ARENA_WIDTH};

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
    }
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

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/pong_spritesheet.ron", // Here we load the associated ron file
        SpriteSheetFormat,
        texture_id, // We pass it the ID of the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.translation.z = 1.0;
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            ARENA_HEIGHT,
            0.0,
        )))
        .with(transform)
        .build();
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
        })
        .with(left_transform)
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
        })
        .with(right_transform)
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
        })
        .with(local_transform)
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
        -50.,
        1.,
        200.,
        50.,
        0,
    );

    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        50.,
        -50.,
        1.,
        200.,
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
        ))
        .build();
    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        ))
        .build();
    world.add_resource(ScoreText { p1_score, p2_score });
}
