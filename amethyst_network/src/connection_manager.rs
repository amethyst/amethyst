use specs::{System,FetchMut};
use shrev::{ReaderId,EventChannel};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use super::{NetSourcedEvent,NetConnectionPool,NetSendBuffer,NetEvent,ConnectionState,NetConnection,NetReceiveBuffer};

pub struct ConnectionManagerSystem<T> where T: PartialEq{
    net_event_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    is_server: bool,
}

impl<T> ConnectionManagerSystem<T> where T: PartialEq{
    pub fn new(is_server: bool) -> Self{
        ConnectionManagerSystem{
            net_event_reader: None,
            is_server,
        }
    }
}

impl<'a,T> System<'a> for ConnectionManagerSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+PartialEq+'static{
    type SystemData = (
        FetchMut<'a, NetReceiveBuffer<T>>,
        FetchMut<'a, NetConnectionPool>,
        FetchMut<'a, NetSendBuffer<T>>,
    );
    fn run(&mut self, (mut events, mut pool, mut send_buf): Self::SystemData) {
        if self.net_event_reader.is_none(){
            self.net_event_reader = Some(events.buf.register_reader());
        }

        for ev in events.buf.read(self.net_event_reader.as_mut().unwrap()){
            if self.is_server{
                match ev.event {
                    NetEvent::Connect => {
                        // Received packet from unknown client
                        if ev.uuid.is_none(){
                            println!("conn manager received something");
                            pool.connections.push(
                                NetConnection{
                                    target: ev.socket,
                                    state: ConnectionState::Connected,
                                    uuid: Uuid::new_v4(),
                                }
                            );
                            send_buf.buf.single_write(
                                NetSourcedEvent{
                                    event: NetEvent::Connected{
                                        server_uuid: Uuid::new_v4(), // TODO RETURN SELF.UUID
                                    },
                                    uuid: None,
                                    socket: ev.socket,
                                }
                            );
                        }
                    },
                    NetEvent::Disconnect{ref reason} => {
                        if let Some(conn) = pool.remove_connection_for_address(&ev.socket){
                            send_buf.buf.single_write(
                                NetSourcedEvent{
                                    event: NetEvent::Disconnected{
                                        reason: reason.clone(),
                                    },
                                    uuid: None,
                                    socket: conn.target,
                                }
                            );
                        }
                    },
                    _ => {},
                }
            }else{
                match ev.event {
                    NetEvent::Connected{server_uuid} => {
                        let mut conn = pool.connection_from_address(&ev.socket);
                        if let Some(mut c) = conn.as_mut(){
                            c.state = ConnectionState::Connected;
                            c.uuid = server_uuid;
                        }
                        info!("Remote {:?} accepted connection request.", ev.socket);
                    },
                    NetEvent::ConnectionRefused { ref reason } => {
                        pool.remove_connection_for_address(&ev.socket);
                        info!("Connection refused by server: {}", reason);
                    },
                    NetEvent::Disconnected { ref reason } => {
                        pool.remove_connection_for_address(&ev.socket);
                        info!("Disconnected from server: {}", reason);
                    },
                    _ => {},
                }
            }
        }
    }
}