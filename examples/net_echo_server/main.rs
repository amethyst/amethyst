extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::ecs::{Fetch, FetchMut, System};
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
    let game = Application::build("", State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_bundle(NetworkBundle::<()>::new_server(
            "127.0.0.1",
            Some(3456 as u16),
            vec![Box::new(FilterConnected::<()>::new())],
        ))?
        .with(EchoSystem::new(), "echo", &[]);

    Ok(game.build()?.run())
}

/// Default empty state.
pub struct State1;
impl State for State1 {}

/// The echo system sends any received event to all connected clients.
struct EchoSystem {
    pub reader: Option<ReaderId<NetSourcedEvent<()>>>,
}

impl EchoSystem {
    pub fn new() -> Self {
        EchoSystem { reader: None }
    }
}

impl<'a> System<'a> for EchoSystem {
    type SystemData = (
        FetchMut<'a, NetReceiveBuffer<()>>,
        FetchMut<'a, NetSendBuffer<()>>,
        Fetch<'a, NetConnectionPool>,
    );
    fn run(&mut self, (mut rcv, mut send, pool): Self::SystemData) {
        if self.reader.is_none() {
            self.reader = Some(rcv.buf.register_reader());
        }
        for ev in rcv.buf.read(self.reader.as_mut().unwrap()) {
            send_to_all(ev.event.clone(), &mut send, &pool);
        }
    }
}
