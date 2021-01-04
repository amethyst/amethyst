//! This module holds the underlying system implementations for each of the various transport
//! protocols. One important thing to note if you're implementing your own, the underlying sockets
//! MUST be non-blocking in order to play nicely with the ECS scheduler.

pub mod laminar;
pub mod tcp;
pub mod udp;

use std::{collections::VecDeque, net::SocketAddr};

use crate::simulation::{
    message::Message,
    requirements::{DeliveryRequirement, UrgencyRequirement},
};

/// Resource serving as the owner of the queue of messages to be sent. This resource also serves
/// as the interface for other systems to send messages.
pub struct TransportResource {
    messages: VecDeque<Message>,
    frame_budget_bytes: i32,
    latency_nanos: i64,
    packet_loss: f32,
}

impl TransportResource {
    /// Creates a new `TransportResource`.
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            frame_budget_bytes: 0,
            latency_nanos: 0,
            packet_loss: 0.0,
        }
    }

    /// Returns estimated number of bytes you can reliably send this frame.
    pub fn frame_budget_bytes(&self) -> i32 {
        self.frame_budget_bytes
    }

    /// Sets the frame budget in bytes. This should be called by a transport implementation.
    pub fn set_frame_budget_bytes(&mut self, budget: i32) {
        self.frame_budget_bytes = budget;
    }

    /// Returns the estimated millisecond round-trip latency for messages.
    pub fn latency_millis(&mut self) -> i64 {
        self.latency_nanos / 1_000_000
    }

    /// Returns the estimated microsecond round-trip latency for messages.
    pub fn latency_micros(&mut self) -> i64 {
        self.latency_nanos / 1000
    }

    /// Returns the estimated nanosecond round-trip latency for messages.
    pub fn latency_nanos(&self) -> i64 {
        self.latency_nanos
    }

    /// Sets the latency value. This should be called by a transport implementation.
    pub fn set_latency_nanos(&mut self, latency: i64) {
        self.latency_nanos = latency;
    }

    /// Returns the estimated loss percentage of packets in 0.0-1.0.
    pub fn packet_loss(&self) -> f32 {
        self.packet_loss
    }

    /// Sets the packet loss value. This should be called by a transport implementation.
    pub fn set_packet_loss(&mut self, loss: f32) {
        self.packet_loss = loss;
    }

    /// Creates a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on next sim tick.
    pub fn send(&mut self, destination: SocketAddr, payload: &[u8]) {
        self.send_with_requirements(
            destination,
            payload,
            DeliveryRequirement::Default,
            UrgencyRequirement::OnTick,
        );
    }

    /// Creates a `Message` with the default guarantees provided by the `Socket` implementation and
    /// Pushes it onto the messages queue to be sent immediately.
    pub fn send_immediate(&mut self, destination: SocketAddr, payload: &[u8]) {
        self.send_with_requirements(
            destination,
            payload,
            DeliveryRequirement::Default,
            UrgencyRequirement::Immediate,
        );
    }

    /// Creates and queue a `Message` with the specified guarantee.
    pub fn send_with_requirements(
        &mut self,
        destination: SocketAddr,
        payload: &[u8],
        delivery: DeliveryRequirement,
        timing: UrgencyRequirement,
    ) {
        let message = Message::new(destination, payload, delivery, timing);
        self.messages.push_back(message);
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Returns a reference to the owned messages.
    pub fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Returns the messages to send by returning the immediate messages or anything adhering to
    /// the given filter.
    pub fn drain_messages_to_send(
        &mut self,
        mut filter: impl FnMut(&mut Message) -> bool,
    ) -> Vec<Message> {
        self.drain_messages(|message| {
            message.urgency == UrgencyRequirement::Immediate || filter(message)
        })
    }

    /// Drains the messages queue and returns the drained messages. The filter allows you to drain
    /// only messages that adhere to your filter. This might be useful in a scenario like draining
    /// messages with a particular urgency requirement.
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

impl Default for TransportResource {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            frame_budget_bytes: 0,
            latency_nanos: 0,
            packet_loss: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_with_default_requirements() {
        let mut resource = create_test_resource();

        resource.send("127.0.0.1:3000".parse().unwrap(), test_payload());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.delivery, DeliveryRequirement::Default);
        assert_eq!(packet.urgency, UrgencyRequirement::OnTick);
    }

    #[test]
    fn test_send_immediate_message() {
        let mut resource = create_test_resource();

        resource.send_immediate("127.0.0.1:3000".parse().unwrap(), test_payload());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.delivery, DeliveryRequirement::Default);
        assert_eq!(packet.urgency, UrgencyRequirement::Immediate);
    }

    #[test]
    fn test_has_messages() {
        let mut resource = create_test_resource();
        assert_eq!(resource.has_messages(), false);
        resource.send_immediate("127.0.0.1:3000".parse().unwrap(), test_payload());
        assert_eq!(resource.has_messages(), true);
    }

    #[test]
    fn test_drain_only_immediate_messages() {
        let mut resource = create_test_resource();

        let addr = "127.0.0.1:3000".parse().unwrap();
        resource.send_immediate(addr, test_payload());
        resource.send_immediate(addr, test_payload());
        resource.send(addr, test_payload());
        resource.send(addr, test_payload());
        resource.send_immediate(addr, test_payload());

        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 3);
        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 0);
    }

    #[test]
    fn test_drain_only_messages_with_specific_requirements() {
        let mut resource = create_test_resource();

        let addr = "127.0.0.1:3000".parse().unwrap();
        resource.send_with_requirements(
            addr,
            test_payload(),
            DeliveryRequirement::Unreliable,
            UrgencyRequirement::OnTick,
        );
        resource.send_with_requirements(
            addr,
            test_payload(),
            DeliveryRequirement::Reliable,
            UrgencyRequirement::OnTick,
        );
        resource.send_with_requirements(
            addr,
            test_payload(),
            DeliveryRequirement::ReliableOrdered(None),
            UrgencyRequirement::OnTick,
        );
        resource.send_with_requirements(
            addr,
            test_payload(),
            DeliveryRequirement::ReliableSequenced(None),
            UrgencyRequirement::OnTick,
        );
        resource.send_with_requirements(
            addr,
            test_payload(),
            DeliveryRequirement::Unreliable,
            UrgencyRequirement::OnTick,
        );

        assert_eq!(
            resource
                .drain_messages(|message| message.delivery == DeliveryRequirement::Unreliable)
                .len(),
            2
        );
        // validate removal
        assert_eq!(
            resource
                .drain_messages(|message| message.delivery == DeliveryRequirement::Unreliable)
                .len(),
            0
        );
    }

    #[test]
    fn test_send_with_requirements() {
        use DeliveryRequirement::*;
        let mut resource = create_test_resource();
        let addr = "127.0.0.1:3000".parse().unwrap();

        let requirements = [
            Unreliable,
            UnreliableSequenced(None),
            Reliable,
            ReliableSequenced(None),
            ReliableOrdered(None),
            Default,
        ];

        for req in requirements.iter().cloned() {
            resource.send_with_requirements(addr, test_payload(), req, UrgencyRequirement::OnTick);
        }

        assert_eq!(resource.messages.len(), requirements.len());

        for (i, req) in requirements.iter().enumerate() {
            assert_eq!(resource.messages[i].delivery, *req);
        }
    }

    fn test_payload() -> &'static [u8] {
        b"test"
    }

    fn create_test_resource() -> TransportResource {
        TransportResource::new()
    }
}
