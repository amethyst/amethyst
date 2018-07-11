
#[cfg(test)]
mod test{
  use amethyst_core::specs::{World,RunNow,VecStorage,ReadStorage};
  use amethyst_core::specs::storage::UnprotectedStorage;
  use ::{NetSocketSystem,NetConnection,NetEvent};
  use amethyst_core::shred::DispatcherBuilder;
  use std::net::{IpAddr,SocketAddr};
  use std::str::FromStr;
  use std::thread::sleep;
  use std::time::Duration;
  
  #[test]
  fn single_packet(){
    let mut world_cl = World::new();
    let mut cl_dispatch = DispatcherBuilder::new()
        .with(NetSocketSystem::<()>::new("127.0.0.1",21201).unwrap(), "s", &[])
        .build();
    cl_dispatch.setup(&mut world_cl.res);
    let mut conn_to_server = NetConnection::<()>::new(
        SocketAddr::new(IpAddr::from_str("::1").unwrap(),21200)
    );
    let test_event = NetEvent::TextMessage{msg: "Test Message From Client1".to_string()};
    conn_to_server.send_buffer.single_write(test_event.clone());
    world_cl.create_entity().with(conn_to_server).build();
    
    let mut world_sv = World::new();
    let mut sv_dispatch = DispatcherBuilder::new()
        .with(NetSocketSystem::<()>::new("127.0.0.1",21200).unwrap(), "s", &[])
        .build();
    sv_dispatch.setup(&mut world_sv.res);
    let mut conn_to_client = NetConnection::<()>::new(
        SocketAddr::new(IpAddr::from_str("::1").unwrap(),21201)
    );
    let mut rcv = conn_to_client.receive_buffer.register_reader();
    let conn_to_client_entity = world_sv.create_entity().with(conn_to_client).build();

    cl_dispatch.dispatch(&mut world_cl.res);
    sleep(Duration::from_millis(100));
    sv_dispatch.dispatch(&mut world_sv.res);
    
    let sto = world_sv.read_storage::<NetConnection<()>>();
    let comp = sto.get(conn_to_client_entity).unwrap();
    //assert!(world_sv.read_storage::<NetConnection<()>>().get(conn_to_client_entity).unwrap().receive_buffer.read(&mut rcv).next() == Some(&test_event));
    assert!(comp.receive_buffer.read(&mut rcv).count() == 1);
  }
}
