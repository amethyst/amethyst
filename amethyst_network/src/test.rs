#[cfg(test)]
mod test {
    use std::{net::SocketAddr, thread::sleep, time::Duration};

    use amethyst_core::{
        shred::{Dispatcher, DispatcherBuilder, SystemData},
        specs::{Builder, Join, World, WriteStorage},
    };

    use crate::{NetConnection, NetEvent, NetSocketSystem};

    #[test]
    fn single_packet_early() {
        let addr1: SocketAddr = "127.0.0.1:21200".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:21201".parse().unwrap();
        let (mut world_cl, mut cl_dispatch, mut world_sv, mut sv_dispatch) =
            build(addr1.clone(), addr2.clone());

        let mut conn_to_server = NetConnection::<()>::new(addr2);
        let mut conn_to_client = NetConnection::<()>::new(addr1);

        let test_event = NetEvent::TextMessage {
            msg: "1".to_string(),
        };
        conn_to_server.send_buffer.single_write(test_event.clone());
        world_cl.create_entity().with(conn_to_server).build();

        let mut rcv = conn_to_client.receive_buffer.register_reader();
        let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

        cl_dispatch.dispatch(&mut world_cl.res);
        sleep(Duration::from_millis(1000));
        sv_dispatch.dispatch(&mut world_sv.res);

        let storage = world_sv.read_storage::<NetConnection<()>>();
        let comp = storage.get(conn_to_client_entity).unwrap();

        assert_eq!(comp.receive_buffer.read(&mut rcv).next(), Some(&test_event));
        // We should have consumed the only event in the iterator by calling next().
        assert!(comp.receive_buffer.read(&mut rcv).count() == 0);
    }

    #[test]
    fn send_receive_100_packets() {
        let addr1: SocketAddr = "127.0.0.1:21205".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:21204".parse().unwrap();
        let (mut world_cl, mut cl_dispatch, mut world_sv, mut sv_dispatch) =
            build(addr1.clone(), addr2.clone());

        let conn_to_server = NetConnection::<()>::new(addr2);
        let mut conn_to_client = NetConnection::<()>::new(addr1);

        let test_event = NetEvent::TextMessage {
            msg: "Test Message From Client1".to_string(),
        };

        world_cl.create_entity().with(conn_to_server).build();

        let mut rcv = conn_to_client.receive_buffer.register_reader();
        let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

        sleep(Duration::from_millis(50));
        {
            let mut sto = WriteStorage::<NetConnection<()>>::fetch(&world_cl.res);

            for cmp in (&mut sto).join() {
                for _i in 0..100 {
                    cmp.send_buffer.single_write(test_event.clone());
                }
            }
        }
        cl_dispatch.dispatch(&mut world_cl.res);
        sleep(Duration::from_millis(500));
        sv_dispatch.dispatch(&mut world_sv.res);
        let storage = world_sv.read_storage::<NetConnection<()>>();
        let comp = storage.get(conn_to_client_entity).unwrap();
        assert_eq!(comp.receive_buffer.read(&mut rcv).count(), 100);
    }

    fn build<'a, 'b>(
        addr1: SocketAddr,
        addr2: SocketAddr,
    ) -> (World, Dispatcher<'a, 'b>, World, Dispatcher<'a, 'b>) {
        let mut world_cl = World::new();
        let mut world_sv = World::new();

        let mut cl_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new(addr1, Vec::new()).unwrap(),
                "s",
                &[],
            )
            .build();
        cl_dispatch.setup(&mut world_cl.res);
        let mut sv_dispatch = DispatcherBuilder::new()
            .with(
                NetSocketSystem::<()>::new(addr2, Vec::new()).unwrap(),
                "s",
                &[],
            )
            .build();
        sv_dispatch.setup(&mut world_sv.res);

        (world_cl, cl_dispatch, world_sv, sv_dispatch)
    }
}
