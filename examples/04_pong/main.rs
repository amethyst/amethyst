//! TODO: Rewrite for new renderer.

extern crate amethyst;

mod pong;
mod systems;
mod bundle;
mod audio;

use std::time::Duration;

use amethyst::Result;
use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::ecs::audio::AudioBundle;
use amethyst::input::InputBundle;
use amethyst::ecs::rendering::{create_render_system, RenderBundle};
use amethyst::ecs::transform::{Transform, TransformBundle};
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::prelude::*;
use amethyst::util::frame_limiter::FrameRateLimitStrategy;

use audio::Music;
use bundle::PongBundle;

const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;
const PADDLE_HEIGHT: f32 = 15.0;
const PADDLE_WIDTH: f32 = 2.5;
const PADDLE_VELOCITY: f32 = 75.0;
const PADDLE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.5;
const BALL_COLOUR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

const AUDIO_MUSIC: &'static [&'static str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &'static str = "audio/bounce.ogg";
const AUDIO_SCORE: &'static str = "audio/score.ogg";

type DrawFlat = pass::DrawFlat<PosTex, Transform>;

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use pong::Pong;

    let display_config_path = format!(
        "{}/examples/04_pong/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let display_config = DisplayConfig::load(display_config_path);

    let key_bindings_path = format!(
        "{}/examples/04_pong/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::new()),
    );

    let game = Application::build(assets_dir, Pong)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_bundle(PongBundle)?
        .with_bundle(TransformBundle::new().with_dep(&["ball_system", "paddle_system"]))?
        .with_bundle(AudioBundle::new(|music: &mut Music| music.music.next()))?
        .with_bundle(RenderBundle::new())?
        .with_local(create_render_system(pipe, Some(display_config))?);
    Ok(game.build()?.run())
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
