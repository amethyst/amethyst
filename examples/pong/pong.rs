use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::Vector3;
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::ecs::prelude::World;
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, Event, PngFormat, Projection, Sprite, SpriteRenderData, Texture, TextureHandle,
    VirtualKeyCode, WindowMessages, WithSpriteRender,
};
use amethyst::ui::{Anchor, TtfFormat, UiText, UiTransform};
use systems::ScoreText;
use {Ball, Paddle, Side};
use {ARENA_HEIGHT, ARENA_WIDTH, SPRITESHEET_SIZE};

pub struct Pong;

impl<'a, 'b> State<GameData<'a, 'b>> for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        use audio::initialise_audio;

        // Load the spritesheet necessary to render the graphics.
        let spritesheet = {
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

        // Setup our game.
        initialise_paddles(world, spritesheet.clone());
        initialise_ball(world, spritesheet);
        initialise_camera(world);
        initialise_audio(world);
        initialise_score(world);
        hide_cursor(world);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::{Matrix4, Vector3};
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            ARENA_HEIGHT,
            0.0,
        )))
        .with(GlobalTransform(
            Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into(),
        ))
        .build();
}

/// Hide the cursor, so it's invisible while playing.
fn hide_cursor(world: &mut World) {
    use amethyst::winit::CursorState;

    world
        .write_resource::<WindowMessages>()
        .send_command(|win| {
            if let Err(err) = win.set_cursor_state(CursorState::Hide) {
                eprintln!("Unable to make cursor hidden! Error: {:?}", err);
            }
        });
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World, spritesheet: TextureHandle) {
    use {PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = (ARENA_HEIGHT - PADDLE_HEIGHT) / 2.0;
    left_transform.translation = Vector3::new(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.translation = Vector3::new(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Create a left plank entity.
    let paddle_left = world
        .create_entity()
        .with(Paddle {
            side: Side::Left,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        })
        .with(GlobalTransform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    let paddle_right = world
        .create_entity()
        .with(Paddle {
            side: Side::Right,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        })
        .with(GlobalTransform::default())
        .with(right_transform)
        .build();

    // Build the sprite for the paddles.
    let sprite = Sprite {
        left: 0.0,
        right: PADDLE_WIDTH,
        top: 0.0,
        bottom: PADDLE_HEIGHT,
    };

    // Add the sprite to the paddles.
    // This is done in a separate step here because as they are identical sprites,
    // adding them both at once allows for better performance. Check out the ball
    // sprite to see how to add a sprite to only one entity.
    world
        .exec(|mut data: SpriteRenderData| {
            data.add_multiple(
                vec![paddle_left, paddle_right],
                &sprite,
                spritesheet,
                SPRITESHEET_SIZE,
            )
        })
        .expect("Error creating SpriteRender for paddles");
}

/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, spritesheet: TextureHandle) {
    use {ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y, PADDLE_WIDTH};

    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.translation = Vector3::new(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Create the sprite for the ball.
    let sprite = Sprite {
        left: PADDLE_WIDTH,
        right: SPRITESHEET_SIZE.0,
        top: 0.0,
        bottom: BALL_RADIUS * 2.0,
    };

    world
        .create_entity()
        .with_sprite(&sprite, spritesheet, SPRITESHEET_SIZE)
        .expect("Error creating SpriteRender for ball")
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .with(GlobalTransform::default())
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
