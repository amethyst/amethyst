#[cfg(test)]
mod test {
    use amethyst_core::shred::{DispatcherBuilder, SystemData};
    use amethyst_core::specs::storage::UnprotectedStorage;
    use amethyst_core::specs::{Join, ReadStorage, RunNow, VecStorage, World, WriteStorage,Builder};
    use fern::Dispatch;
    use log::LevelFilter;
    use std::io;
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;
    use std::thread::sleep;
    use std::time::Duration;
    use {NetConnection, NetEvent, NetSocketSystem};

    #[test]
    fn single_packet_early() {
        let mut world_cl = World::new();
        let mut world_sv = World::new();

        let mut cl_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new("127.0.0.1", 21201).unwrap(),
                "s",
                &[],
            )
            .build();
        cl_dispatch.setup(&mut world_cl.res);
        let mut sv_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new("127.0.0.1", 21200).unwrap(),
                "s",
                &[],
            )
            .build();
        sv_dispatch.setup(&mut world_sv.res);

        let mut conn_to_server = NetConnection::<()>::new(SocketAddr::new(
            IpAddr::from_str("127.0.0.1").unwrap(),
            21200,
        ));
        let mut conn_to_client = NetConnection::<()>::new(SocketAddr::new(
            IpAddr::from_str("127.0.0.1").unwrap(),
            21201,
        ));

        //let test_event = NetEvent::TextMessage{msg: "Test Message From Client1".to_string()};
        let test_event = NetEvent::TextMessage {
            msg: "1".to_string(),
        };
        conn_to_server.send_buffer.single_write(test_event.clone());
        world_cl.create_entity().with(conn_to_server).build();

        let mut rcv = conn_to_client.receive_buffer.register_reader();
        let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

        cl_dispatch.dispatch(&mut world_cl.res);
        sv_dispatch.dispatch(&mut world_sv.res);

        let storage = world_sv.read_storage::<NetConnection<()>>();
        let comp = storage.get(conn_to_client_entity).unwrap();

        assert_eq!(comp.receive_buffer.read(&mut rcv).next(), Some(&test_event));
        // We should have consumed the only event in the iterator by calling next().
        assert!(comp.receive_buffer.read(&mut rcv).count() == 0);
    }
    #[test]
    fn send_receive_100k_packets() {
        Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(LevelFilter::Debug)
            .chain(io::stdout())
            .apply()
            .unwrap_or_else(|_| {
                debug!("Global logger already set, default amethyst logger will not be used")
            });

        let mut world_cl = World::new();
        let mut world_sv = World::new();

        let mut cl_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new("127.0.0.1", 21201).unwrap(),
                "s",
                &[],
            )
            .build();
        cl_dispatch.setup(&mut world_cl.res);
        let mut sv_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new("127.0.0.1", 21200).unwrap(),
                "s",
                &[],
            )
            .build();
        sv_dispatch.setup(&mut world_sv.res);

        let mut conn_to_server = NetConnection::<()>::new(SocketAddr::new(
            IpAddr::from_str("127.0.0.1").unwrap(),
            21200,
        ));
        let mut conn_to_client = NetConnection::<()>::new(SocketAddr::new(
            IpAddr::from_str("127.0.0.1").unwrap(),
            21201,
        ));

        let test_event = NetEvent::TextMessage {
            msg: "Test Message From Client1".to_string(),
        };
        /*for i in 0..100000{
      conn_to_server.send_buffer.single_write(test_event.clone());
    }*/
        world_cl.create_entity().with(conn_to_server).build();

        let mut rcv = conn_to_client.receive_buffer.register_reader();
        let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

        for i in 0..10 {
            sleep(Duration::from_millis(50));
            {
                let mut sto = WriteStorage::<NetConnection<()>>::fetch(&world_cl.res);
                for mut cmp in (&mut sto).join() {
                    for i in 0..10000 {
                        cmp.send_buffer.single_write(test_event.clone());
                    }
                }
            }
            cl_dispatch.dispatch(&mut world_cl.res);
            sleep(Duration::from_millis(10));
            sv_dispatch.dispatch(&mut world_sv.res);
            let storage = world_sv.read_storage::<NetConnection<()>>();
            let comp = storage.get(conn_to_client_entity).unwrap();
            assert_eq!(comp.receive_buffer.read(&mut rcv).count(), 10000);
        }

        /*let storage = world_sv.read_storage::<NetConnection<()>>();
    let comp = storage.get(conn_to_client_entity).unwrap();
    assert_eq!(comp.receive_buffer.read(&mut rcv).count(), 100000);*/
    }
}
