use laminar::{error::NetworkError, Event, Packet};

/// Net event which occurred on the network.
pub enum ServerSocketEvent {
    /// event containing a packet with received data
    Packet(Packet),
    /// event containing an error that has occurred in the network
    Error(NetworkError),
    /// events that can happen with a client
    ClientEvent(ClientEvent),
    /// Event used for a default initialisation.
    Empty,
}

/// Event that could occur with a client.
pub enum ClientEvent {
    /// represents a connecting client
    Connected,
    /// represents a disconnecting client
    Disconnected,
    /// represents a client who is timing out
    Timedout,
    /// represents an default value for this enum.
    QualityChange,
}

/// Convert a `laminar` client event to our own client event.
impl From<Event> for ClientEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Connected(_) => ClientEvent::Connected,
            Event::Disconnected(_) => ClientEvent::Disconnected,
            Event::TimedOut(_) => ClientEvent::Timedout,
            Event::QualityChange { .. } => ClientEvent::QualityChange,
        }
    }
}
