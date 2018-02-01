extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::ecs::{FetchMut, System};
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::ReaderId;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // We make the server run at 1 fps, to see how much the buffer can handle before we lose events
    let game = Application::build("", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            1,
        )
        .with_bundle(NetworkBundle::<()>::new_server(
            "127.0.0.1",
            Some(3456 as u16),
            vec![Box::new(FilterConnected::<()>::new())],
        ))?
        .with(SpamReceiveSystem::new(), "rcv", &[]);

    Ok(game.build()?.run())
}

/// Default empty state
pub struct State1;
impl State for State1 {}

/// A simple system that receives a ton of network events.
struct SpamReceiveSystem {
    pub reader: Option<ReaderId<NetSourcedEvent<()>>>,
}

impl SpamReceiveSystem {
    pub fn new() -> Self {
        SpamReceiveSystem { reader: None }
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (FetchMut<'a, NetReceiveBuffer<()>>,);
    fn run(&mut self, (mut rcv,): Self::SystemData) {
        if self.reader.is_none() {
            self.reader = Some(rcv.buf.register_reader());
        }
        let mut count = 0;
        for ev in rcv.buf.read(self.reader.as_mut().unwrap()) {
            count += 1;
            match ev.event {
                NetEvent::TextMessage { ref msg } => println!("{}", msg),
                _ => {}
            }
        }
        println!("Received {} messages this frame", count);
    }
}
