//! Opens an empty window.

extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::{Context, ContextConfig};
use amethyst::context::event::{Event, VirtualKeyCode};
use amethyst::config::Element;
use amethyst::ecs::World;

struct Example;

impl State for Example {
    fn handle_events(&mut self, events: &[Event], _: &mut Context, _: &mut World) -> Trans {
        for e in events {
            match *e {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }

    fn on_start(&mut self, ctx: &mut Context, _: &mut World) {
        use amethyst::renderer::pass::Clear;
        use amethyst::renderer::Layer;
        let clear_layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                        ]);
        let pipeline = vec![clear_layer];
        ctx.renderer.set_pipeline(pipeline);
    }

    fn update(&mut self, ctx: &mut Context, _: &mut World) -> Trans {
        ctx.renderer.submit();
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/01_window/resources/config.yml",
                        env!("CARGO_MANIFEST_DIR"));
	let config = ContextConfig::from_file(path).unwrap(); 
    let ctx = Context::new(config);
    let mut game = Application::build(Example, ctx).done();
    game.run();
}
