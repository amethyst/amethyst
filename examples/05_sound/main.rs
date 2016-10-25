extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::ecs::World;

struct Example;

impl State for Example {
    fn on_start(&mut self, ctx: &mut Context, _: &mut World) {
        // Uncomment the following code and replace "sound.ogg"
        // with your sound filename in .wav or .ogg format to
        // play it.

        // let path = format!("{}/examples/05_sound/resources/sound.ogg",
        //                    env!("CARGO_MANIFEST_DIR"));
        // ctx.asset_manager.load_sound("sound", path.as_str());
        // let sound = ctx.asset_manager.get_sound("sound").unwrap();
        // if let Some(ref mut sink) = ctx.audio_sink {
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
    use amethyst::context::ContextConfig;
    let config = ContextConfig::default();
    let ctx = Context::new(config);
    let mut game = Application::build(Example, ctx).done();
    game.run();
}
