extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::ecs::World;

struct Example;

impl State for Example {
    fn on_start(&mut self, context: &mut Context, _: &mut World) {
        // Uncomment the following code and replace "sound.ogg"
        // with your sound filename in .wav or .ogg format to
        // play it.

        // context.asset_manager.load_sound("sound", "sound.ogg");
        // let sound = context.asset_manager.get_sound("sound").unwrap();
        // if let Some(ref mut sink) = context.audio_sink {
        //     sink.append(sound);
        // }
    }

    fn update(&mut self, _: &mut Context, _: &mut World) -> Trans {
        use std::io::Read;
        println!("Press [Enter] to quit.");
        // Wait for input and quit
        std::io::stdin().bytes().next();
        Trans::Quit
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::default();
    let mut game = Application::build(Example, config).done();
    game.run();
}
