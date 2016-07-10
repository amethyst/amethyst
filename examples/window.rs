extern crate amethyst;

use std::cell::RefCell;
use std::rc::Rc;
use amethyst::context::event_handler::EventIter;

use amethyst::engine::{Application, Duration, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;

struct Example {
    context: Rc<RefCell<Context>>,
}

impl Example {
    pub fn new(context: Rc<RefCell<Context>>) -> Example {
        Example { context: context }
    }
}

impl State for Example {
    fn handle_events(&mut self, _events: EventIter) -> Trans {
        use amethyst::context::event_handler::{Event, VirtualKeyCode};
        let mut trans = Trans::None;
        for event in _events {
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => trans = Trans::Quit,
                Event::Closed => trans = Trans::Quit,
                _ => (),
            }
        }
        trans
    }

    fn update(&mut self, _delta: Duration) -> Trans {
        use amethyst::context::video_context::VideoContext;
        let context = self.context.borrow_mut();
        match context.video_context {
            VideoContext::OpenGL { ref window, .. } => {
                window.swap_buffers().unwrap();
            }
#[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
            },
        }
        Trans::None
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::from_file("../config/window_example_config.yml").unwrap();
    let context = Context::new(config).unwrap();
    let context_ref = Rc::new(RefCell::new(context));
    let example = Example::new(context_ref.clone());
    let mut game = Application::new(example, context_ref.clone());
    game.run();
}
