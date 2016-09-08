extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Join};

struct Example;

impl State for Example {
    fn on_start(&mut self, ctx: &mut Context, _: &mut World) {
        use amethyst::renderer::pass::Clear;
        use amethyst::renderer::Layer;
        let clear_layer =
            Layer::new("main",
                        vec![
                            Clear::new([0., 0., 0., 1.]),
                        ]);
        let pipeline = vec![clear_layer];
        ctx.renderer.set_pipeline(pipeline);
    }

    fn update(&mut self, ctx: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let engine_events = ctx.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        ctx.renderer.submit();
        Trans::None
    }
}

fn main() {
    use amethyst::context::Config;
	let config = Config::from_file(
        format!("{}/config/window_example_config.yml",
                env!("CARGO_MANIFEST_DIR"))
        ).unwrap(); 
    let ctx = Context::new(config);
    let mut game = Application::build(Example, ctx).done();
    game.run();
}
