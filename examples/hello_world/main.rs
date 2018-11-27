//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::prelude::*;

struct Example;

impl<S, E> StateCallback<S, E> for Example {
    fn on_start(&mut self, _: &mut World) {
        println!("Begin!");
    }

    fn on_stop(&mut self, _: &mut World) {
        println!("End!");
    }

    fn update(&mut self, _: &mut World) -> Trans<S> {
        println!("Hello from Amethyst!");
        Trans::Quit
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let mut game = Application::build("./")?
        .with_state((), Example)?
        .build(GameDataBuilder::default())?;

    game.run();
    Ok(())
}
