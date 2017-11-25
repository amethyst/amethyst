extern crate amethyst;

mod pong;

use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosNormTex, RenderBundle, RenderSystem,
                         Stage};


fn run() -> Result<(), amethyst::Error> {
    use pong::Pong;

    let path = format!("{}/examples/pong_tutorial_01/resources/display_config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(Stage::with_backbuffer()
        .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
        .with_pass(DrawFlat::<PosNormTex>::new()));

    let mut game = Application::build("./", Pong)
        ?
        .with_bundle(RenderBundle::new())?
        .with_local(RenderSystem::build(pipe, Some(config))?)
        .build()
        .expect("Fatal error");

    game.run();

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Error occurred during game execution: {}", e);
        ::std::process::exit(1);
    }
}
