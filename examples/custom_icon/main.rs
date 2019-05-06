//! Opens an empty window.

use amethyst::{
    input::is_key_down,
    prelude::*,
    renderer::{DisplayConfig, DrawFlat, Pipeline, PosNormTex, RenderBundle, Stage},
    utils::application_root_dir,
    winit::{Icon, VirtualKeyCode},
};

struct Example;

impl SimpleState for Example {
    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = application_root_dir()?.join("examples/custom_icon/resources/display_config.ron");
    let mut config = DisplayConfig::load(&path);
    let mut icon = Vec::new();
    for _ in 0..(128 * 128) {
        icon.extend(vec![255, 0, 0, 255]);
    }
    config.loaded_icon = Some(Icon::from_rgba(icon, 128, 128).unwrap());

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosNormTex, f32>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::<'_, _, _, f32>::new(pipe, Some(config)))?;
    let mut game = Application::new("./", Example, game_data)?;

    game.run();

    Ok(())
}
