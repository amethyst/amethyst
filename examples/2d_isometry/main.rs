extern crate amethyst;

use amethyst::controls::XYCameraSystem;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{
    ColorMask, DepthMode, DisplayConfig, DrawSprite, Pipeline, RenderBundle, Stage, ALPHA,
};
use std::time::Duration;

mod camera;
mod cars;
mod map;
mod state;
mod tiles;

use state::IsometryState;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/examples/2d_isometry/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let input_config_path = format!(
        "{}/examples/2d_isometry/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawSprite::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(input_config_path)?,
        )?
        .with_bundle(
            RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&["transform_system"]),
        )?
        .with(
            XYCameraSystem::<String, String>::new("horizontal", "vertical").with_zoom("zoom"),
            "camera_movement",
            &[],
        )
        .with(cars::MoveCars, "cars_system", &[]);
    let mut game = Application::build(assets_dir, IsometryState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
