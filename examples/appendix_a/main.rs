//! TODO: Rewrite for new renderer.

extern crate amethyst;
#[macro_use]
extern crate serde_derive;

mod audio;
mod bundle;
mod config;
mod pong;
mod systems;

use amethyst::{
    audio::AudioBundle,
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    ecs::prelude::{Component, DenseVecStorage},
    input::InputBundle,
    prelude::*,
    renderer::{DrawFlat, PosTex},
    ui::UiBundle,
    utils::application_root_dir,
};
use crate::{audio::Music, bundle::PongBundle, config::PongConfig};
use std::time::Duration;

const AUDIO_MUSIC: &'static [&'static str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &'static str = "audio/bounce.ogg";
const AUDIO_SCORE: &'static str = "audio/score.ogg";

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use crate::pong::Pong;

    let app_root = application_root_dir();

    let display_config_path = format!("{}/examples/appendix_a/resources/display.ron", app_root);
    let key_bindings_path = format!("{}/examples/appendix_a/resources/input.ron", app_root);

    let config = format!("{}/examples/appendix_a/resources/config.ron", app_root);
    let assets_dir = format!("{}/examples/assets/", app_root);

    let pong_config = PongConfig::load(&config);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(PongBundle::default())?
        .with_bundle(TransformBundle::new().with_dep(&["ball_system", "paddle_system"]))?
        .with_bundle(AudioBundle::new(|music: &mut Music| music.music.next()))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawFlat::<PosTex>::new(), true)?;

    let mut game = Application::build(assets_dir, Pong)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with_resource(pong_config.arena)
        .with_resource(pong_config.ball)
        .with_resource(pong_config.paddles)
        .build(game_data)?;

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
