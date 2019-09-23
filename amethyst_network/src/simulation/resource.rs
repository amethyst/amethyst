use super::{
    client::Client,
    message::Message,
    requirements::{DeliveryRequirement, UrgencyRequirement},
    transport::socket::Socket,
};
use std::{collections::VecDeque, net::SocketAddr};

/// Resource serving as the owner of the underlying socket and the queue of messages to be sent.
pub struct NetworkSimulationResource<S: Socket> {
    socket: Option<S>,
    is_server: bool,
    server_addr: Option<SocketAddr>,
    clients: Vec<Client>,
    messages: VecDeque<Message>,
}

impl<S: Socket> NetworkSimulationResource<S> {
    /// Create a new `NetworkSimulationResource` as a client
    pub fn new_client(server_addr: SocketAddr) -> Self {
        Self {
            socket: None,
            server_addr: Some(server_addr),
            clients: Vec::new(),
            messages: VecDeque::new(),
            is_server: false,
        }
    }

    /// Create a new `NetworkSimulationResource` as a server
    pub fn new_server() -> Self {
        Self {
            socket: None,
            server_addr: None,
            clients: Vec::new(),
            messages: VecDeque::new(),
            is_server: true,
        }
    }

    /// Add a number of trusted clients to the `NetworkSimulationResource` for use as a server
    pub fn with_trusted_clients(mut self, clients: &[SocketAddr]) -> Self {
        self.clients = clients.iter().map(|addr| Client::new(*addr)).collect();
        self
    }

    /// Returns a slice of the tracked clients
    pub fn clients(&self) -> &[Client] {
        &self.clients
    }

    /// Set the server address
    pub fn set_server_addr(&mut self, server_addr: Option<SocketAddr>) {
        self.server_addr = server_addr;
    }

    /// Return a mutable reference to the socket if there is one configured.
    pub fn get_socket_mut(&mut self) -> Option<&mut S> {
        self.socket.as_mut()
    }

    /// Set the bound socket to the `NetworkSimulationResource`
    pub fn set_socket(&mut self, socket: S) {
        self.socket = Some(socket);
    }

    /// Drops the socket from the `NetworkSimulationResource`
    pub fn drop_socket(&mut self) {
        self.socket = None;
    }

    /// Returns whether or not the `NetworkSimulationResource` has a bound socket
    pub fn has_socket(&self) -> bool {
        self.socket.is_some()
    }

    /// Returns the server address if one was set
    pub fn server_addr(&self) -> Option<SocketAddr> {
        self.server_addr
    }

    /// Returns whether or not this `NetworkSimulationResource` was created as a 'server'
    pub fn is_server(&self) -> bool {
        self.is_server
    }

    /// Returns whether or not this `NetworkSimulationResource` was created as a 'client'
    pub fn is_client(&self) -> bool {
        !self.is_server
    }

    /// Create a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on next sim tick.
    pub fn send(&mut self, payload: &[u8]) {
        self.send_with_requirements(
            payload,
            S::default_requirement(),
            UrgencyRequirement::OnTick,
        );
    }

    /// Create a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent immediately.
    pub fn send_immediate(&mut self, payload: &[u8]) {
        self.send_with_requirements(
            payload,
            S::default_requirement(),
            UrgencyRequirement::Immediate,
        );
    }

    /// Create and queue a `Message` with the specified guarantee
    pub fn send_with_requirements(
        &mut self,
        payload: &[u8],
        delivery: DeliveryRequirement,
        timing: UrgencyRequirement,
    ) {
        let message = Message::new(payload, delivery, timing);
        self.messages.push_back(message);
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Drains the messages queue and returns the drained messages
    pub fn drain_messages(&mut self, mut filter: impl FnMut(&mut Message) -> bool) -> Vec<Message> {
        let mut drained = Vec::with_capacity(self.messages.len());
        let mut i = 0;
        while i != self.messages.len() {
            if filter(&mut self.messages[i]) {
                if let Some(m) = self.messages.remove(i) {
                    drained.push(m);
                }
            } else {
                i += 1;
            }
        }
        drained
    }
}

impl<S: Socket> Default for NetworkSimulationResource<S> {
    fn default() -> Self {
        panic!(
            "The `NetworkSimulationResource` resource MUST be created and added to the `Application` \
             before use."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::transport::laminar::LaminarSocket;

    #[test]
    fn test_send() {
        let mut net = create_test_resource();
        net.send(test_payload());
        assert_eq!(net.messages.len(), 1);
        let packet = &net.messages[0];
        // Default guarantee specified by the Laminar impl
        assert_eq!(packet.delivery, DeliveryRequirement::ReliableOrdered(None));
        assert_eq!(packet.urgency, UrgencyRequirement::OnTick);
    }

    #[test]
    fn test_send_with_requirements() {
        use DeliveryRequirement::*;
        let mut net = create_test_resource();

        let requirements = [
            Unreliable,
            UnreliableSequenced(None),
            Reliable,
            ReliableSequenced(None),
            ReliableOrdered(None),
        ];

        for req in requirements.iter().cloned() {
            net.send_with_requirements(test_payload(), req, UrgencyRequirement::OnTick);
        }

        assert_eq!(net.messages.len(), requirements.len());

        for (i, req) in requirements.iter().enumerate() {
            assert_eq!(net.messages[i].delivery, *req);
        }
    }

    #[test]
    fn test_has_socket_and_with_socket() {
        let mut net = create_test_resource();
        assert!(!net.has_socket());
        net.set_socket(LaminarSocket::bind_any().unwrap());
        assert!(net.has_socket());
    }

    fn test_payload() -> &'static [u8] {
        b"test"
    }

    fn create_test_resource() -> NetworkSimulationResource<LaminarSocket> {
        let addr = "127.0.0.1:3000".parse().unwrap();
        <NetworkSimulationResource<LaminarSocket>>::new_client(addr)
    }
}
