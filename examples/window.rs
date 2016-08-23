extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Join};

struct Example;

impl State for Example {
    fn update(&mut self, context: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};

        let engine_events = context.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }

    fn on_start(&mut self, context: &mut Context, _: &mut World) {
        use amethyst::renderer::pass::Clear;
        use amethyst::renderer::Layer;
        let clear_layer =
            Layer::new("main",
                        vec![
                            Clear::new([0., 0., 0., 1.]),
                        ]);
        let pipeline = vec![clear_layer];
        context.renderer.set_pipeline(pipeline);
    }
}

fn main() {
    use amethyst::context::ContextConfig;
	  let config = ContextConfig::from_file(
        format!("{}/config/window_example_config.yml",
                env!("CARGO_MANIFEST_DIR"))
        ).unwrap();
    let context = Context::new(config);
    let mut game = Application::build(Example, context).done();
    game.run();
}
