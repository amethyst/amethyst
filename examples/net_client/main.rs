extern crate amethyst;

use std::time::Duration;

use amethyst::Result;
use amethyst::audio::AudioBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, RenderSystem,
                         Stage};
use amethyst::ui::{DrawUi, UiBundle};
use amethyst::network::*;
use amethyst::shrev::EventChannel;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut client = NetClientSystem::<NetEvent>::new("127.0.0.1",4545 as u16).expect("Failed to create NetClientSystem");
    client.connect(SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(),4546 as u16));

    let game = Application::build("",State1)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with(client,"net_client_system",&[])
        .with_resource(EventChannel::<NetOwnedEvent<NetEvent>>::new());

    Ok(
        game.build()?.run(),
    )
}

pub struct State1;

impl State for State1 {
}