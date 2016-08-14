extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Entity};

struct Example;

impl State for Example {
    fn handle_events(&mut self, events: Vec<Entity>, context: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let mut trans = Trans::None;
        let storage = context.broadcaster.read::<EngineEvent>();
        for _event in events {
            let event = storage.get(_event).unwrap();
            let event = &event.payload;
            match *event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => trans = Trans::Quit,
                Event::Closed => trans = Trans::Quit,
                _ => (),
            }
        }
        trans
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

    fn update(&mut self, context: &mut Context, _: &mut World) -> Trans {
        context.renderer.submit();
        Trans::None
    }
}

fn main() {
    use amethyst::context::ContextConfig;
    let config = ContextConfig::from_file("../config/window_example_config.yml").unwrap();
    let context = Context::new(config);
    let mut game = Application::build(Example, context).done();
    game.run();
}
