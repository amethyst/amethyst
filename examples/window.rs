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
    fn on_start(&mut self) {
        println!("Begin!");
    }

    fn update(&mut self, _delta: Duration) -> Trans {
        let window = &mut self.context.borrow_mut().window;
        let mut trans = Trans::None;
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => trans = Trans::Quit,
                _ => ()
            }
        }
        window.swap_buffers().unwrap();
        trans
    }

    fn on_stop(&mut self) {
        println!("End!");
    }
}

fn main() {
    let config = amethyst::context::Config::from_file("../config/window_example_config.yml").unwrap();
    let context = Rc::new(RefCell::new(Context::new(config)));
    let mut game = Application::new(Example::new(context.clone()));
    game.run();
}
