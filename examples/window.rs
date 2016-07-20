extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{Simulation, World, Entity};

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
        use amethyst::context::video_context::VideoContext;
        use amethyst::renderer::pass::*;
        use amethyst::renderer::Layer;
        match context.video_context {
            VideoContext::OpenGL { ref mut frame, .. } => {
                let clear_layer =
                    Layer::new("main",
                               vec![
                                   Clear::new([0., 0., 0., 1.]),
                               ]);
                frame.layers.push(clear_layer);
            }
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
    }

    fn update(&mut self, context: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::video_context::VideoContext;
        match context.video_context {
            VideoContext::OpenGL { ref window,
                                   ref mut renderer,
                                   ref frame,
                                   ref mut device,
                                   ..} => {
                renderer.submit(frame, device);
                window.swap_buffers().unwrap();
            }
#[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
            VideoContext::Null => (),
        }
        Trans::None
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::from_file("../config/window_example_config.yml").unwrap();
    let world = World::new();
    let simulation = Simulation::new(world, 1);
    let mut game = Application::new(Example, simulation, config);
    game.run();
}
