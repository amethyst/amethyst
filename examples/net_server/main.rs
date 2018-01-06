extern crate amethyst;

use std::time::Duration;

use amethyst::Result;
use amethyst::audio::AudioBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, RenderSystem,
                         Stage};
use amethyst::ui::{DrawUi, UiBundle};
use amethyst::network::network_server::*;

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let game = Application::build("",State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with(NetServerSystem::new(),"net_server_system",&[]);

    Ok(
        game.build()?.run(),
    )
}

pub struct State1;

impl State for State1 {
}