extern crate amethyst;

use std::time::Duration;

use amethyst::Result;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::prelude::*;
use amethyst::network::*;
use amethyst::shrev::EventChannel;

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let server = NetServerSystem::<()>::new("127.0.0.1",4546 as u16).expect("Failed to create NetServerSystem");

    let game = Application::build("",State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with(server,"net_server_system",&[])
        .with_resource(EventChannel::<NetOwnedEvent<NetEvent<()>>>::new());

    Ok(
        game.build()?.run(),
    )
}

pub struct State1;

impl State for State1 {
}