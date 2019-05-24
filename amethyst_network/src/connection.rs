//! Network Connection and states.

use serde::{Deserialize, Serialize};
use shrev::{EventChannel, EventIterator, ReaderId};
use std::net::SocketAddr;
use uuid::Uuid;

use amethyst_core::ecs::{Component, VecStorage};

use crate::NetEvent;

// TODO: Think about relationship between NetConnection and NetIdentity.

/// A remote connection connection to some endpoint.
///
/// This type is a `Component`, and it is used by systems too:
/// - Queue received data into this type
/// - Read the queued data for transmission and send it.
///
/// # Remark
/// Note that this type does not perform any reading or writing, this is done only withing systems.
/// This type acts as a container for to send and received data.
#[derive(Serialize)]
#[serde(bound = "")]
pub struct NetConnection<E: 'static> {
    /// The target remote socket address who is listening for incoming packets.
    pub target_addr: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
    // The buffer of events to be sent.
    #[serde(skip)]
    send_buffer: EventChannel<NetEvent<E>>,
    /// The buffer of events that have been received.
    #[serde(skip)]
    pub(crate) receive_buffer: EventChannel<NetEvent<E>>,
    /// The buffer used by `NetSocketSystem` that allows it to immediately send events upon receiving a new `NetConnection`.
    #[serde(skip)]
    send_reader: ReaderId<NetEvent<E>>,
}

impl<E: Send + Sync + 'static> NetConnection<E> {
    /// Construct a new `NetConnection`.
    ///
    /// - `SocketAddr`: the remote enpoint, from here the data will be send to and received from.
    pub fn new(target_addr: SocketAddr) -> Self {
        let mut send_buffer = EventChannel::new();
        let send_reader = send_buffer.register_reader();

        NetConnection {
            target_addr,
            state: ConnectionState::Connecting,
            send_buffer,
            receive_buffer: EventChannel::<NetEvent<E>>::new(),
            send_reader,
        }
    }

    /// This function is used ONLY by `NetSocketSystem`.
    ///
    /// Most users both create the connection and send messages on the same frame,
    /// we need a way to read those. Since the `NetSocketSystem` runs after the creation of the NetConnection,
    /// it cannot possibly have registered his reader early enough to catch the initial messages that the user wants to send.
    ///
    /// The downside of this is that you are forced to take NetConnection mutably inside of NetSocketSystem.
    /// If someone finds a better solution, please open a PR.
    pub(crate) fn send_buffer_early_read(&mut self) -> EventIterator<'_, NetEvent<E>> {
        self.send_buffer.read(&mut self.send_reader)
    }

    /// Queues the given event iterator of events for transmission.
    pub fn queue_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = NetEvent<E>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.send_buffer.iter_write(iter);
    }

    /// Queues the given vector of events for transmission.
    pub fn queue_vec(&mut self, events: &mut Vec<NetEvent<E>>) {
        self.send_buffer.drain_vec_write(events);
    }

    /// Queues a single event for transmission.
    pub fn queue(&mut self, event: NetEvent<E>) {
        self.send_buffer.single_write(event);
    }

    /// Returns an iterator over the received events.
    ///
    /// - `reader_id`: the reader id of the registered reader.
    ///
    /// # Remark
    /// - To be able to read events, a reader id is required. This is required for the underlying ringbuffer.
    /// The reader id  stores information of where in the ringbuffer the reader has read from earlier.
    /// Please checkout `register_reader` which will return a `ReaderId` that allows you to read the received events.
    pub fn received_events(
        &self,
        reader_id: &mut ReaderId<NetEvent<E>>,
    ) -> EventIterator<NetEvent<E>> {
        self.receive_buffer.read(reader_id)
    }

    /// Returns a `ReaderId` that can be used to read from the received events.
    ///
    /// # Remark
    /// - To be able to read events, a reader id is required. This is because otherwise the channel
    /// wouldn't know where in the ringbuffer the reader has read to earlier. This information is
    /// stored in the reader id.
    pub fn register_reader(&mut self) -> ReaderId<NetEvent<E>> {
        self.receive_buffer.register_reader()
    }
}

impl<E> PartialEq for NetConnection<E> {
    fn eq(&self, other: &Self) -> bool {
        self.target_addr == other.target_addr
    }
}

impl<E: PartialEq> Eq for NetConnection<E> {}

impl<E: Send + Sync + 'static> Component for NetConnection<E> {
    type Storage = VecStorage<Self>;
}

///The state of the connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// The connection is established.
    Connected,
    /// The connection is being established.
    Connecting,
    /// The connection has been dropped.
    Disconnected,
}

/// A network identity. It can represent either a client or a server.
/// It represents anything that can own an entity or a component.
/// Think of it as an identity card.
/// When used as a resource, it designates the local network uuid.
pub struct NetIdentity {
    /// The uuid identifying this NetIdentity.
    pub uuid: Uuid,
}

impl Default for NetIdentity {
    fn default() -> Self {
        NetIdentity {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Component for NetIdentity {
    type Storage = VecStorage<NetIdentity>;
}

#[cfg(test)]
mod tests {
    use crate::connection::NetConnection;
    use crate::net_event::NetEvent;

    #[test]
    fn can_register_reader() {
        let mut connection = test_connection();
        connection
            .receive_buffer
            .single_write(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));

        let reader_id = connection.register_reader();
    }

    fn can_read_received_events() {
        let mut connection = test_connection();
        connection
            .receive_buffer
            .single_write(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));

        let mut reader_id = connection.register_reader();
        assert_eq!(connection.received_events(&mut reader_id).len(), 1);
    }

    fn can_queue_packet_for_send() {
        let mut connection = test_connection();
        connection.queue(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));

        assert_eq!(connection.send_buffer_early_read().len(), 1);
    }

    fn can_queue_vec_for_send() {
        let mut connection = test_connection();

        let mut to_send_data = Vec::new();
        to_send_data.push(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));
        to_send_data.push(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));
        to_send_data.push(NetEvent::Connected("127.0.0.1:0".parse().unwrap()));

        connection.queue_vec(&mut to_send_data);

        assert_eq!(
            connection
                .send_buffer
                .read(&mut connection.send_reader)
                .len(),
            3
        );
    }

    fn test_connection() -> NetConnection<String> {
        return NetConnection::new("127.0.0.1:0".parse().unwrap());
    }
}
