extern crate amethyst;

use std::cell::RefCell;
use std::rc::Rc;

use amethyst::engine::{Application, Duration, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::Entity;

struct Example {
    context: Rc<RefCell<Context>>,
}

impl Example {
    pub fn new(context: Rc<RefCell<Context>>) -> Example {
        Example {
            context: context,
        }
    }
}

impl State for Example {
    fn handle_events(&mut self, events: Vec<Entity>) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let mut trans = Trans::None;
        let context = self.context.borrow_mut();
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

    fn on_start(&mut self) {
        use amethyst::context::video_context::VideoContext;
        use amethyst::renderer::pass::*;
        use amethyst::renderer::Layer;
        let mut context = self.context.borrow_mut();
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

    fn update(&mut self, _delta: Duration) -> Trans {
        use amethyst::context::video_context::VideoContext;
        let mut context = self.context.borrow_mut();
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
    let context = Context::new(config);
    let context_ref = Rc::new(RefCell::new(context));
    let example = Example::new(context_ref.clone());
    let mut game = Application::new(example, context_ref.clone());
    game.run();
}
