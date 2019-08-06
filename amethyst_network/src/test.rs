#![cfg(test)]

use std::{net::SocketAddr, thread::sleep, time::Duration};

use amethyst_core::{
    ecs::{Builder, Join, World, WorldExt, WriteStorage},
    shred::{Dispatcher, DispatcherBuilder, SystemData},
};

use crate::{
    net_event::{NetEvent, NetPacket},
    server::ServerConfig,
    NetConnection, NetSocketSystem,
};
use laminar::Config;

#[test]
fn single_packet_early() {
    let server_addr: SocketAddr = "127.0.0.1:21200".parse().unwrap();
    let client_addr: SocketAddr = "127.0.0.1:21202".parse().unwrap();

    let (mut world_cl, mut cl_dispatch, mut world_sv, mut sv_dispatch) =
        build(client_addr, server_addr);

    let mut conn_to_server = NetConnection::<String>::new(server_addr);
    let mut conn_to_client = NetConnection::<String>::new(client_addr);

    let packet = NetEvent::Packet(NetPacket::reliable_unordered(
        "Test Message From Client1".to_string(),
    ));

    conn_to_server.queue(packet.clone());
    world_cl.create_entity().with(conn_to_server).build();

    let mut rcv = conn_to_client.receive_buffer.register_reader();
    let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

    cl_dispatch.dispatch(&world_cl);
    sleep(Duration::from_millis(500));
    sv_dispatch.dispatch(&world_sv);

    let storage = world_sv.read_storage::<NetConnection<String>>();
    let comp = storage.get(conn_to_client_entity).unwrap();

    assert_eq!(comp.receive_buffer.read(&mut rcv).next(), Some(&packet));
    // We should have consumed the only event in the iterator by calling next().
    assert!(comp.receive_buffer.read(&mut rcv).count() == 0);
}

#[test]
#[ignore]
fn send_receive_100_packets() {
    let server_addr: SocketAddr = "127.0.0.1:21204".parse().unwrap();
    let client_addr: SocketAddr = "127.0.0.1:21206".parse().unwrap();

    // setup world for client and server
    let (mut world_cl, mut cl_dispatch, mut world_sv, mut sv_dispatch) =
        build(client_addr, server_addr);

    // setup connections from client -> server and server -> client
    let conn_to_server = NetConnection::<String>::new(server_addr);
    let mut conn_to_client = NetConnection::<String>::new(client_addr);

    let packet = NetEvent::Packet(NetPacket::reliable_unordered(
        "Test Message From Client1".to_string(),
    ));

    world_cl.create_entity().with(conn_to_server).build();

    let mut rcv = conn_to_client.receive_buffer.register_reader();
    let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

    sleep(Duration::from_millis(50));
    {
        let mut sto = WriteStorage::<NetConnection<String>>::fetch(&world_cl);

        for cmp in (&mut sto).join() {
            for _i in 0..100 {
                cmp.queue(packet.clone());
            }
        }
    }
    cl_dispatch.dispatch(&world_cl);
    sleep(Duration::from_millis(100));
    sv_dispatch.dispatch(&world_sv);

    let storage = world_sv.read_storage::<NetConnection<String>>();
    let comp = storage.get(conn_to_client_entity).unwrap();
    assert_eq!(comp.receive_buffer.read(&mut rcv).count(), 100);
}

fn build<'a, 'b>(
    client_addr: SocketAddr,
    server_addr: SocketAddr,
) -> (World, Dispatcher<'a, 'b>, World, Dispatcher<'a, 'b>) {
    let mut world_cl = World::new();
    let mut world_sv = World::new();

    // client config
    let client_config = ServerConfig {
        udp_socket_addr: client_addr,
        max_throughput: 10000,
        create_net_connection_on_connect: false,
        laminar_config: Config::default(),
    };

    // server config
    let server_config = ServerConfig {
        udp_socket_addr: server_addr,
        max_throughput: 10000,
        create_net_connection_on_connect: false,
        laminar_config: Config::default(),
    };

    let mut cl_dispatch = DispatcherBuilder::new()
        .with(
            NetSocketSystem::<String>::new(client_config).unwrap(),
            "s",
            &[],
        )
        .build();
    cl_dispatch.setup(&mut world_cl);
    let mut sv_dispatch = DispatcherBuilder::new()
        .with(
            NetSocketSystem::<String>::new(server_config).unwrap(),
            "s",
            &[],
        )
        .build();
    sv_dispatch.setup(&mut world_sv);

    (world_cl, cl_dispatch, world_sv, sv_dispatch)
}
