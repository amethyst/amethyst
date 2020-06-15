// CLIENT
use std::time::Duration;

use amethyst::{
    core::{bundle::SystemBundle, frame_limiter::FrameRateLimitStrategy, SystemDesc, Time},
    ecs::{DispatcherBuilder, Read, System, SystemData, World, Write},
    network::simulation::{
        tcp::TcpNetworkBundle, NetworkSimulationEvent, NetworkSimulationTime, TransportResource,
    },
    prelude::*,
    shrev::{EventChannel, ReaderId},
    utils::application_root_dir,
    Result,
};
use log::{error, info};

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("examples/net_client/");

    //    // UDP
    //    let socket = UdpSocket::bind("0.0.0.0:3455")?;
    //    socket.set_nonblocking(true)?;

    //    // TCP: No listener needed for the client.

    //    // Laminar
    //    let socket = LaminarSocket::bind("0.0.0.0:3455")?;

    let game_data = GameDataBuilder::default()
        //        // UDP
        //        .with_bundle(UdpNetworkBundle::new(Some(socket), 2048))?
        // TCP
        .with_bundle(TcpNetworkBundle::new(None, 2048))?
        //        // Laminar
        //        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with_bundle(SpamBundle)?;

    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

/// Default empty state
pub struct GameState;
impl SimpleState for GameState {}

#[derive(Debug)]
struct SpamBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for SpamBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(SpamSystemDesc::default().build(world), "spam_system", &[]);
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct SpamSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, SpamSystem> for SpamSystemDesc {
    fn build(self, world: &mut World) -> SpamSystem {
        // Creates the EventChannel<NetworkEvent> managed by the ECS.
        <SpamSystem as System<'_>>::SystemData::setup(world);
        // Fetch the change we just created and call `register_reader` to get a
        // ReaderId<NetworkEvent>. This reader id is used to fetch new events from the network event
        // channel.
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        SpamSystem::new(reader)
    }
}

/// A simple system that receives a ton of network events.
struct SpamSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl SpamSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (
        Read<'a, NetworkSimulationTime>,
        Read<'a, Time>,
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
    );
    fn run(&mut self, (sim_time, time, mut net, event /*, tx*/): Self::SystemData) {
        let server_addr = "127.0.0.1:3457".parse().unwrap();
        for frame in sim_time.sim_frames_to_run() {
            info!("Sending message for sim frame {}.", frame);
            let payload = format!(
                "CL: sim_frame:{},abs_time:{}",
                frame,
                time.absolute_time_seconds()
            );
            net.send(server_addr, payload.as_bytes());
        }

        for event in event.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => info!("Payload: {:?}", payload),
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => info!("Server Disconnected: {}", addr),
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => {}
            }
        }
    }
}
