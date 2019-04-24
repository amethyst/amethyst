//! A simple headless server

use amethyst::prelude::*;

struct Headless;

impl EmptyState for Headless {
    fn on_start(&mut self, _: StateData<'_, ()>) {
        let server_addr: SocketAddr = "127.0.0.1:25200".parse().unwrap();
        println!("Begin!");
    }

    fn on_stop(&mut self, _: StateData<'_, ()>) {
        println!("End!");
    }

    fn update(&mut self, _: StateData<'_, ()>) -> EmptyTrans {
        println!("Hello from Headless Amethyst!");
        Trans::Quit
    }
}

fn main() {
    amethyst::start_logger(Default::default());
    let mut game = Application::new("./", Headless, ()).expect("Fatal error");
    game.run();
}
