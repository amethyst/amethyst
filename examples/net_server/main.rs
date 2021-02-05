// SERVER
use std::{net::TcpListener, time::Duration};

use amethyst::{
    core::{
        ecs::{System, SystemBundle},
        frame_limiter::FrameRateLimitStrategy,
    },
    network::simulation::{
        tcp::TcpNetworkBundle, NetworkSimulationEvent, NetworkSimulationTime, TransportResource,
    },
    prelude::*,
    shrev::{EventChannel, ReaderId},
    utils::application_root_dir,
    Result,
};
use log::{error, info};
use systems::ParallelRunnable;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let listener = TcpListener::bind("0.0.0.0:3457")?;
    listener.set_nonblocking(true)?;

    let assets_dir = application_root_dir()?.join("examples/net_server");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(TcpNetworkBundle::new(Some(listener), 2048))
        .add_bundle(SpamReceiveBundle);

    let game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

pub struct GameState;
impl SimpleState for GameState {}

#[derive(Debug)]
struct SpamReceiveBundle;

impl SystemBundle for SpamReceiveBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<()> {
        let mut chan = EventChannel::<NetworkSimulationEvent>::default();
        let reader = chan.register_reader();
        resources.insert(chan);

        resources.insert(TransportResource::default());
        resources.insert(NetworkSimulationTime::default());

        builder.add_system(SpamReceiveSystem { reader });
        Ok(())
    }
}

/// A simple system that receives a ton of network events.
#[derive(Debug)]
struct SpamReceiveSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl System for SpamReceiveSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("SpamReceiveSystem")
                .read_resource::<EventChannel<NetworkSimulationEvent>>()
                .write_resource::<TransportResource>()
                .build(move |_commands, _world, (channel, net), _query| {
                    for event in channel.read(&mut self.reader) {
                        match event {
                            NetworkSimulationEvent::Message(addr, payload) => {
                                info!("{}: {:?}", addr, payload);
                                // In a typical client/server simulation, both the client and the server will
                                // be exchanging messages at a constant rate. Laminar makes use of this by
                                // packaging message acks with the next sent message. Therefore, in order for
                                // reliability to work properly, we'll send a generic "ok" response.
                                net.send(*addr, b"ok");
                            }
                            NetworkSimulationEvent::Connect(addr) => {
                                info!("New client connection: {}", addr)
                            }
                            NetworkSimulationEvent::Disconnect(addr) => {
                                info!("Client Disconnected: {}", addr);
                            }
                            NetworkSimulationEvent::RecvError(e) => {
                                error!("Recv Error: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                }),
        )
    }
}
