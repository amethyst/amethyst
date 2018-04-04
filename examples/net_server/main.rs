extern crate amethyst;
#[macro_use]
extern crate log;

use std::time::Duration;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel,ReaderId};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use amethyst::ecs::{Fetch, FetchMut, System};
use amethyst::network::NetworkClientBundle;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // We make the server run at 5 fps, to see how much the buffer can handle before we lose events
    let game = Application::build("", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            1,
        )
        .with_bundle(NetworkClientBundle::<()>::new(
            "127.0.0.1",
            Some(3456 as u16),
            vec![Box::new(FilterConnected)],
            true,
        ))?
        .with(SpamReceiveSystem::new(),"rcv",&[]);

    Ok(game.build()?.run())
}

pub struct State1;

impl State for State1 {}


struct SpamReceiveSystem{
    pub reader: Option<ReaderId<NetSourcedEvent<()>>>,
}

impl SpamReceiveSystem{
    pub fn new() -> Self{
        SpamReceiveSystem{
            reader: None,
        }
    }
}

impl<'a> System<'a> for SpamReceiveSystem {
    type SystemData = (
        FetchMut<'a, NetReceiveBuffer<()>>,
    );
    fn run(&mut self, (mut rcv,): Self::SystemData) {
        if self.reader.is_none() {
            self.reader = Some(rcv.buf.register_reader());
        }
        let mut count = 0;
        for ev in rcv.buf.read(self.reader.as_mut().unwrap()) {
            count += 1;
            match ev.event{
                NetEvent::TextMessage {ref msg} => println!("{}",msg),
                _ => {},
            }
        }
        println!("Received {} messages this frame",count);
    }
}
