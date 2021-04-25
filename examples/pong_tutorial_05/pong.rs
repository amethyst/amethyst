use amethyst::{
    assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
    core::{Time, transform::Transform},
    prelude::*,
    renderer::{sprite::Sprites, Camera, SpriteRender, SpriteSheet, Texture},
    ui::{Anchor, LineMode, UiText, UiTransform},
};

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

#[derive(Default)]
pub struct Pong {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        // Wait one second before spawning the ball.
        self.ball_spawn_timer.replace(1.0);

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle
            .replace(load_sprite_sheet(resources));
        initialize_paddles(world, self.sprite_sheet_handle.clone().unwrap());
        initialize_camera(world);
        initialize_scoreboard(world, resources);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData {
            world, resources, ..
        } = data;

        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, substract the time that passed since last update.
            {
                let time = resources.get::<Time>().unwrap();
                timer -= time.delta_time().as_secs_f32();
            }
            if timer <= 0.0 {
                // When timer expire, spawn the ball
                initialize_ball(world, self.sprite_sheet_handle.clone().unwrap());
            } else {
                // If timer is not expired yet, put it back onto the state.
                self.ball_spawn_timer.replace(timer);
            }
        }
        Trans::None
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            x: match side {
                Side::Right => ARENA_WIDTH - PADDLE_WIDTH / 2.,
                Side::Left => PADDLE_WIDTH / 2.,
            },
            y: ARENA_HEIGHT / 2.,
        }
    }
}

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

/// ScoreBoard contains the actual score data
#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: i32,
    pub score_right: i32,
}

/// ScoreText contains the ui text elements that display the score
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}

fn load_sprite_sheet(resources: &mut Resources) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `sprite_sheet` is the layout of the sprites on the image
    // `texture_handle` is a cloneable reference to the texture

    let texture_handle: Handle<Texture> = {
        let loader = resources.get::<DefaultLoader>().unwrap();
        loader.load("texture/pong_spritesheet.png")
    };

    let loader = resources.get::<DefaultLoader>().unwrap();
    let sprites: Handle<Sprites> = loader.load(
        "texture/pong_spritesheet.ron", // Here we load the associated ron file
    );
    let sheet = SpriteSheet {
        texture: texture_handle,
        sprites,
    };
    let q = resources.get::<ProcessingQueue<SpriteSheet>>().unwrap();
    loader.load_from_data(sheet, (), &q)
}

/// initialize the camera.
fn initialize_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world.push((Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT), transform));
}

/// initializes one paddle on the left, and one paddle on the right.
fn initialize_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Assign the sprites for the paddles
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 0); // paddle is the first sprite in the sprite_sheet

    // Create a left plank entity.
    world.push((
        sprite_render.clone(),
        Paddle::new(Side::Left),
        left_transform,
    ));

    // Create right plank entity.
    world.push((sprite_render, Paddle::new(Side::Right), right_transform));
}

/// initializes one ball in the middle of the arena.
fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(
        (ARENA_WIDTH - BALL_RADIUS) * 0.5,
        (ARENA_HEIGHT - BALL_RADIUS) * 0.5,
        0.0,
    );

    // Assign the sprite for the ball
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 1); // ball is the second sprite on the sprite_sheet

    world.push((
        sprite_render,
        Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        },
        local_transform,
    ));
}

/// initializes a ui scoreboard
fn initialize_scoreboard(world: &mut World, resources: &mut Resources) {
    resources.insert(ScoreBoard::default());

    let font = {
        let loader = resources.get::<DefaultLoader>().unwrap();
        loader.load("font/square.ttf")
    };

    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        -50.,
        -50.,
        1.,
        200.,
        50.,
    );
    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        50.,
        -50.,
        1.,
        200.,
        50.,
    );

    let p1_score = world.push((
        p1_transform,
        UiText::new(
            Some(font.clone()),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
            LineMode::Single,
            Anchor::Middle,
        ),
    ));

    let p2_score = world.push((
        p2_transform,
        UiText::new(
            Some(font),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
            LineMode::Single,
            Anchor::Middle,
        ),
    ));

    resources.insert(ScoreText { p1_score, p2_score });
}
