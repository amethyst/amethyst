//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate futures;

mod pong;
mod systems;
mod bundle;
mod audio;

use std::time::Duration;

use amethyst::Result;
use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::ecs::audio::DjBundle;
use amethyst::ecs::input::InputBundle;
use amethyst::ecs::rendering::{MaterialComponent, MeshComponent, RenderBundle};
use amethyst::ecs::transform::{Transform, TransformBundle};
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::prelude::*;
use bundle::PongBundle;
use amethyst::util::frame_limiter::FrameRateLimitStrategy;

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
    "Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &'static str = "bounce.ogg";
const AUDIO_SCORE: &'static str = "score.ogg";

type DrawFlat = pass::DrawFlat<PosTex, MeshComponent, MaterialComponent, Transform>;

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use amethyst::assets::Directory;
    use pong::Pong;

    let display_config_path = format!(
        "{}/examples/04_pong/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let key_bindings_path = format!(
        "{}/examples/04_pong/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let assets_dir = format!("{}/examples/04_pong/resources/", env!("CARGO_MANIFEST_DIR"));

    let game = Application::build(Pong)?
        .with_frame_limit(FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)), 144)
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_bundle(PongBundle)?
        .with_bundle(TransformBundle::new().with_dep(&["ball_system", "paddle_system"]))?
        .with_bundle(DjBundle::new())?
        .with_bundle(
            RenderBundle::new(
                Pipeline::build().with_stage(
                    Stage::with_backbuffer()
                        .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                        .with_pass(DrawFlat::new()),
                ),
            ).with_config(DisplayConfig::load(display_config_path)),
        )?
        .with_store("assets", Directory::new(assets_dir));
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
