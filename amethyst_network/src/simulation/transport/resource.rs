use crate::simulation::{
    message::Message,
    requirements::{DeliveryRequirement, UrgencyRequirement},
};
use std::{collections::VecDeque, net::SocketAddr};

/// Resource serving as the owner of the queue of messages to be sent. This resource also serves
/// as the interface for other systems to send messages
pub struct TransportResource {
    messages: VecDeque<Message>,
}

impl TransportResource {
    /// Create a new `TransportResource`
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    /// Create a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on next sim tick.
    pub fn send(&mut self, destination: SocketAddr, payload: &[u8]) {
        self.send_with_requirements(
            destination,
            payload,
            DeliveryRequirement::Default,
            UrgencyRequirement::OnTick,
        );
    }

    /// Create a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent immediately.
    pub fn send_immediate(&mut self, destination: SocketAddr, payload: &[u8]) {
        self.send_with_requirements(
            destination,
            payload,
            DeliveryRequirement::Default,
            UrgencyRequirement::Immediate,
        );
    }

    /// Create and queue a `Message` with the specified guarantee
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
