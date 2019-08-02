//! The simplest Amethyst example.

use amethyst::prelude::*;

struct Example;

impl EmptyState for Example {
    fn on_start(&mut self, _: StateData<'_, ()>) {
        println!("Begin!");
    }

    fn on_stop(&mut self, _: StateData<'_, ()>) {
        println!("End!");
    }

    fn update(&mut self, _: StateData<'_, ()>) -> EmptyTrans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let assets_dir = "./";
    let mut game = Application::new(assets_dir, Example, ())?;
    game.run();

    Ok(())
}
