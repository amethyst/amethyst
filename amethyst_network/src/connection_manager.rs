use super::{ConnectionState, NetConnection, NetConnectionPool, NetEvent, NetIdentity,
            NetReceiveBuffer, NetSendBuffer, NetSourcedEvent};
use serde::Serialize;
use serde::de::DeserializeOwned;
use shrev::{EventChannel, ReaderId};
use specs::{Fetch, FetchMut, System};
use uuid::Uuid;

pub struct ConnectionManagerSystem<T>
where
    T: PartialEq,
{
    net_event_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    is_server: bool,
}

impl<T> ConnectionManagerSystem<T>
where
    T: PartialEq,
{
    pub fn new(is_server: bool) -> Self {
        ConnectionManagerSystem {
            net_event_reader: None,
            is_server,
        }
    }
}

impl<'a, T> System<'a> for ConnectionManagerSystem<T>
where
    T: Send + Sync + PartialEq + 'static,
{
    type SystemData = (
        FetchMut<'a, NetReceiveBuffer<T>>,
        FetchMut<'a, NetConnectionPool>,
        FetchMut<'a, NetSendBuffer<T>>,
        Fetch<'a, NetIdentity>,
    );
    fn run(&mut self, (mut events, mut pool, mut send_buf, identity): Self::SystemData) {
        if self.net_event_reader.is_none() {
            self.net_event_reader = Some(events.buf.register_reader());
        }

        for ev in events.buf.read(self.net_event_reader.as_mut().unwrap()) {
            if self.is_server {
                match ev.event {
                    NetEvent::Connect { client_uuid } => {
                        // Received packet from unknown/disconnected client
                        if ev.uuid.is_none() {
                            // Check if the specified uuid is already connected.
                            // UUID Spoofing prevention.
                            if pool.connections
                                .iter()
                                .filter(|c| match c.uuid {
                                    Some(cl_uuid) => cl_uuid == client_uuid,
                                    None => false,
                                })
                                .count() == 0
                            {
                                pool.connections.push(NetConnection {
                                    target: ev.socket,
                                    state: ConnectionState::Connected,
                                    uuid: Some(client_uuid),
                                });
                                send_buf.buf.single_write(NetSourcedEvent {
                                    event: NetEvent::Connected {
                                        server_uuid: identity.uuid.clone(),
                                    },
                                    uuid: Some(client_uuid),
                                    socket: ev.socket,
                                });
                            }
                        }
                    }
                    NetEvent::Disconnect { ref reason } => {
                        if let Some(conn) = pool.remove_connection_for_address(&ev.socket) {
                            send_buf.buf.single_write(NetSourcedEvent {
                                event: NetEvent::Disconnected {
                                    reason: reason.clone(),
                                },
                                uuid: None,
                                socket: conn.target,
                            });
                        }
                    }
                    _ => {}
                }
            } else {
                match ev.event {
                    NetEvent::Connected { server_uuid } => {
                        let mut conn = pool.connection_from_address(&ev.socket);
                        if let Some(mut c) = conn.as_mut() {
                            c.state = ConnectionState::Connected;
                            c.uuid = Some(server_uuid);
                        }
                        info!("Remote {:?} accepted connection request.", ev.socket);
                    }
                    NetEvent::ConnectionRefused { ref reason } => {
                        pool.remove_connection_for_address(&ev.socket);
                        info!("Connection refused by server: {}", reason);
                    }
                    NetEvent::Disconnected { ref reason } => {
                        pool.remove_connection_for_address(&ev.socket);
                        info!("Disconnected from server: {}", reason);
                    }
                    _ => {}
                }
            }
        }
    }
}
