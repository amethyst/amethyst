//! This module holds the underlying system implementations for each of the various transport
//! protocols. One important thing to note if you're implementing your own, the underlying sockets
//! MUST be non-blocking in order to play nicely with the ECS scheduler.

mod resource;

pub mod laminar;
pub mod socket;
pub mod udp;

pub use crate::simulation::transport::resource::SimulationTransportResource;

const NETWORK_SIM_TIME_SYSTEM_NAME: &str = "simulation_time";
const NETWORK_SEND_SYSTEM_NAME: &str = "network_send";
const NETWORK_RECV_SYSTEM_NAME: &str = "network_recv";
const NETWORK_POLL_SYSTEM_NAME: &str = "network_poll";

use crate::simulation::{
    transport::socket::Socket, Message, NetworkSimulationTime, UrgencyRequirement,
};
use log::warn;

/// Shared set up code for implementations of `NetworkSendSystem`s
pub fn run_network_send_system<T: Socket>(
    net: &mut SimulationTransportResource<T>,
    sim_time: &NetworkSimulationTime,
    mut handle_send: impl FnMut(&mut T, &Message) -> (),
) {
    // If no socket configured, this system should be a no-op.
    if !net.has_socket() {
        if net.has_messages() {
            warn!("Messages waiting to be sent but no socket configured.");
        }
        return;
    }

    let messages = net.drain_messages(|message| {
        message.urgency == UrgencyRequirement::Immediate
            || sim_time.should_send_message_this_frame()
    });

    let socket = net.get_socket_mut().expect("A socket should be configured");
    for message in messages.iter() {
        handle_send(socket, message);
    }
}

/// Shared set up code for implementations of `NetworkRecvSystem`s
pub fn run_network_recv_system<T: Socket>(
    net: &mut SimulationTransportResource<T>,
    mut handle_recv: impl FnMut(&mut T) -> (),
) {
    net.get_socket_mut().map(|socket| handle_recv(socket));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{
        DeliveryRequirement, NetworkSimulationTime, SimulationTransportResource,
    };
    use std::time::Duration;

    // If there are no messages to send, the `handle_send` function shouldn't be invoked
    #[test]
    fn run_network_send_system_no_op() {
        let (mut net, sim_time) = setup_test();
        let mut call_count = 0;
        run_network_send_system(&mut net, &sim_time, |_, _| {
            call_count += 1;
        });
        assert_eq!(call_count, 0);
    }

    // Since we're on the first frame, the callback will be invoked immediately
    #[test]
    fn run_network_send_system_should_call_with_send() {
        let (mut net, sim_time) = setup_test();

        net.send("127.0.0.1:9001".parse().unwrap(), b"test");

        let mut call_count = 0;
        run_network_send_system(&mut net, &sim_time, |_, _| {
            call_count += 1;
        });
        assert_eq!(call_count, 1);
    }

    // On the second frame, with message send rate of 2, the callback will not be invoked.
    #[test]
    fn run_network_send_system_should_not_call_on_next_frame() {
        let (mut net, mut sim_time) = setup_test();

        net.send("127.0.0.1:9001".parse().unwrap(), b"test");

        sim_time.update_elapsed(Duration::from_secs(1));
        sim_time.increment_frame_number();

        let mut call_count = 0;
        run_network_send_system(&mut net, &sim_time, |_, _| {
            call_count += 1;
        });
        assert_eq!(call_count, 0);
    }

    // On the second frame, with message send rate of 2, the callback will be invoked on messages
    // with immediate urgency.
    #[test]
    fn run_network_send_system_should_call_on_next_frame_with_immediate() {
        let (mut net, mut sim_time) = setup_test();

        net.send_immediate("127.0.0.1:9001".parse().unwrap(), b"test");

        sim_time.update_elapsed(Duration::from_secs(1));
        sim_time.increment_frame_number();

        let mut call_count = 0;
        run_network_send_system(&mut net, &sim_time, |_, _| {
            call_count += 1;
        });
        assert_eq!(call_count, 1);
    }

    fn setup_test() -> (
        SimulationTransportResource<TestSocket>,
        NetworkSimulationTime,
    ) {
        let socket = TestSocket {};
        let mut net = SimulationTransportResource::new();
        net.set_socket(socket);

        // Set up the sim_time to send every other frame
        let sim_time = NetworkSimulationTime::default().with_message_send_rate(2);
        (net, sim_time)
    }

    struct TestSocket;

    impl Socket for TestSocket {
        fn default_requirement() -> DeliveryRequirement {
            DeliveryRequirement::Unreliable
        }
    }
}
