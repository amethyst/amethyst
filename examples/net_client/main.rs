extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::Result;
use amethyst::core::Time;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::ecs::{Fetch, FetchMut, System};
use amethyst::network::*;
use amethyst::prelude::*;
use amethyst::shrev::ReaderId;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        error!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let game = Application::build("", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_bundle(
            NetworkBundle::<()>::new(
                "127.0.0.1",
                Some(3455 as u16),
                vec![Box::new(FilterConnected::<()>::new())],
                false,
            ).with_connect(SocketAddr::new(
                IpAddr::from_str("127.0.0.1").unwrap(),
                3456 as u16,
            )),
        )?
        .with(SpamSystem::new(), "spam_system", &[])
        .with(ReaderSystem::new(), "reader", &[]);

    Ok(game.build()?.run())
}

/// Default empty state
pub struct State1;
impl State for State1 {}

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
struct SpamSystem {}

impl SpamSystem {
    pub fn new() -> Self {
        SpamSystem {}
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (
        FetchMut<'a, NetSendBuffer<()>>,
        Fetch<'a, NetConnectionPool>,
        Fetch<'a, Time>,
    );
    fn run(&mut self, (mut send_buf, pool, time): Self::SystemData) {
        for i in 0..10000 {
            let ev = NetEvent::TextMessage {
                msg: format!(
                    "CL: frame:{},abs_time:{},c:{}",
                    time.frame_number(),
                    time.absolute_time_seconds(),
                    i
                ),
            };
            send_to_all(ev, &mut send_buf, &pool);
        }
    }
}

/// A simple system reading received events.
/// Used to see events sent by the net_echo_server example.
struct ReaderSystem {
    pub reader: Option<ReaderId<NetSourcedEvent<()>>>,
}

impl ReaderSystem {
    pub fn new() -> Self {
        ReaderSystem { reader: None }
    }
}

impl<'a> System<'a> for ReaderSystem {
    type SystemData = (FetchMut<'a, NetReceiveBuffer<()>>,);
    fn run(&mut self, (mut rcv,): Self::SystemData) {
        if self.reader.is_none() {
            self.reader = Some(rcv.buf.register_reader());
        }
        for ev in rcv.buf.read(self.reader.as_mut().unwrap()) {
            match ev.event {
                NetEvent::TextMessage { ref msg } => println!("Received: {}", msg),
                _ => {}
            }
        }
    }
}
