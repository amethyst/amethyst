//! Opens an empty window.

extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::{Context, ContextConfig};
use amethyst::config::Element;
use amethyst::ecs::{World, Entity};

struct Example;

impl State for Example {
    fn handle_events(&mut self, events: &[Entity], ctx: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let mut trans = Trans::None;
        let storage = ctx.broadcaster.read::<EngineEvent>();
        for e in events {
            let event = storage.get(*e).unwrap();
            match event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => trans = Trans::Quit,
                Event::Closed => trans = Trans::Quit,
                _ => (),
            }
        }
        trans
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
