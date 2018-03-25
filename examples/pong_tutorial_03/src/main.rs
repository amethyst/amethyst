extern crate amethyst;

mod pong;
mod systems;

use std::time::Duration;

use amethyst::Result;
use amethyst::core::transform::TransformBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::prelude::*;
use amethyst::input::InputBundle;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, RenderSystem,
                         Stage};

fn run() -> Result<()> {
    use pong::Pong;

    let config = DisplayConfig::load("./resources/display_config.ron");

    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file("./resources/bindings_config.ron");

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let mut game = Application::build("./", Pong)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(RenderBundle::new())?
        .with_local(RenderSystem::build(pipe, Some(config))?)
        .with(systems::PaddleSystem, "paddle_sys", &["input_system"])
        .build()?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Error occurred during game execution: {}", e);
        ::std::process::exit(1);
    }
}
