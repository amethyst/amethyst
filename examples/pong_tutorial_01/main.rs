extern crate amethyst;

use amethyst::input::{is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::{DrawFlat, Event, PosTex, VirtualKeyCode};

pub struct Pong;

impl<'a, 'b> State<GameData<'a, 'b>> for Pong {
    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    let path = format!(
        "{}/examples/pong_tutorial_01/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    // This line is not mentionned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data =
        GameDataBuilder::default().with_basic_renderer(path, DrawFlat::<PosTex>::new(), false)?;
    let mut game = Application::new(assets_dir, Pong, game_data)?;
    game.run();
    Ok(())
}
