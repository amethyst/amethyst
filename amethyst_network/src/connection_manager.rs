use super::{ConnectionState, NetConnection, NetConnectionPool, NetEvent, NetIdentity,
            NetReceiveBuffer, NetSendBuffer, NetSourcedEvent};
use shrev::ReaderId;
use specs::{Fetch, FetchMut, System};

/// Manages the network connections.
/// The way it is done depends if it is assigned to work as a server or as a client.
///
/// # Standard Events
///
/// Server Mode:
/// Input: Connect
/// Output: Connected
///
/// Input: Disconnect
/// Output: Disconnected
///
/// Clients Mode:
/// Input: Connected
///
/// Input: ConnectionRefused
///
/// Input: Disconnected
///
// TODO: Allow user to specify how uuid are assigned to connections.
pub struct ConnectionManagerSystem<T>
where
    T: PartialEq,
{
    /// The reader for the NetReceiveBuffer.
    net_event_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    /// Indicates how it should handle events and reply to them.
    is_server: bool,
}

impl<T> ConnectionManagerSystem<T>
where
    T: PartialEq,
{
    /// Creates a new ConnectionManagerSystem.
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
                // Server mode
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
                                // Add the connection
                                pool.connections.push(NetConnection {
                                    target: ev.socket,
                                    state: ConnectionState::Connected,
                                    uuid: Some(client_uuid),
                                });
                                // Reply with Connected
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
                            // If the client was connected, we reply that it is Disconnected
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
                // Client mode
                match ev.event {
                    NetEvent::Connected { server_uuid } => {
                        let mut conn = pool.connection_from_address_mut(&ev.socket);
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
