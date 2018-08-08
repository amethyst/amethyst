//! TODO: Rewrite for new renderer.

extern crate amethyst;

mod audio;
mod bundle;
mod pong;
mod systems;

use std::time::Duration;

use amethyst::audio::AudioBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::DrawSprite;
use amethyst::ui::UiBundle;

use audio::Music;
use bundle::PongBundle;

const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;
const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_WIDTH: f32 = 4.0;
const PADDLE_VELOCITY: f32 = 75.0;

const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.0;

const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);

const AUDIO_MUSIC: &'static [&'static str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &'static str = "audio/bounce.ogg";
const AUDIO_SCORE: &'static str = "audio/score.ogg";

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use pong::Pong;

    let display_config_path = format!(
        "{}/examples/pong/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            format!(
                "{}/examples/pong/resources/input_controller.ron",
                env!("CARGO_MANIFEST_DIR")
            )
        } else {
            format!(
                "{}/examples/pong/resources/input.ron",
                env!("CARGO_MANIFEST_DIR")
            )
        }
    };

    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(PongBundle)?
        .with_bundle(TransformBundle::new().with_dep(&["ball_system", "paddle_system"]))?
        .with_bundle(AudioBundle::new(|music: &mut Music| music.music.next()))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawSprite::new(), true)?;
    let mut game = Application::build(assets_dir, Pong)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).build(game_data)?;
    game.run();
    Ok(())
}

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub velocity: f32,
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            velocity: 1.0,
            side: side,
            width: 1.0,
            height: 1.0,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct ScoreBoard {
    score_left: i32,
    score_right: i32,
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            score_left: 0,
            score_right: 0,
        }
    }
}
