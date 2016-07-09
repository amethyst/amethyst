extern crate amethyst;
extern crate glutin;

use std::cell::RefCell;
use std::rc::Rc;

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
    fn update(&mut self, _delta: Duration) -> Trans {
        use amethyst::context::VideoContext;
        match self.context.borrow_mut().video_context {
            VideoContext::OpenGL { ref window, .. } => {
                let mut trans = Trans::None;
                for event in window.poll_events() {
                    match event {
                        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => trans = Trans::Quit,
                        glutin::Event::Closed => trans = Trans::Quit,
                        _ => (),
                    }
                }
                window.swap_buffers().unwrap();
                trans
            },
#[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                Trans::Quit
            },
        }
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::from_file("../config/window_example_config.yml").unwrap();
    let context = Context::new(config).unwrap();
    let context_ref = Rc::new(RefCell::new(context));
    let example = Example::new(context_ref.clone());
    let mut game = Application::new(example);
    game.run();
}
