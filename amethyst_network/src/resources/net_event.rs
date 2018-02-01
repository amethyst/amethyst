//! The network events that are passed from machine to machine, and within the ECS event handling system.
//! NetEvent are passed through the network
//! NetOwnedEvent are passed through the ECS, and contains the event's source (remote connection, usually)

use shrev::Event;
use resources::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

///The basic network events shipped with amethyst
///You can add more event by making a new enum and having this one has a variant of yours.
///
/// ```rust
/// #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
/// pub enum SampleUserEvents{
///     Base(NetEvent),
///     CustomEvent,
/// }
///
/// impl BaseNetEvent<SampleUserEvents> for SampleUserEvents{
///     fn base_to_custom(ev:NetEvent) -> SampleUserEvents {
///         SampleUserEvents::Base(ev)
///     }
///     fn custom_to_base(ev: SampleUserEvents) -> Option<NetEvent> {
///         match ev{
///             SampleUserEvents::Base(e) => Some(e),
///             _=>None,
///         }
///     }
/// }
/// ```
//TODO: Add CreateEntity,RemoveEntity,UpdateEntity
//TODO: Example of switching the NetEvent set in the Network Systems
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum NetEvent<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    /// Ask to connect to the server
    Connect,
    /// Reply to the client that the connection has been accepted
    Connected,
    /// Reply to the client that the connection has been refused
    ConnectionRefused{
        /// The reason of the refusal
        reason:String
    },
    /// Tell the server that the client is disconnecting
    Disconnect{
        /// The reason of the disconnection
        reason:String
    },
    /// Notify the clients(including the one being disconnected) that a client has been disconnected from the server
    Disconnected{
        /// The reason of the disconnection
        reason:String
    },
    Custom(T),
}

impl<T> NetEvent<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    fn custom(&self) -> Option<&T> {
        if let NetEvent::Custom(T) = self {
            Some(&T)
        } else {
            None
        }
    }
}

/*impl BaseNetEvent<NetEvent> for NetEvent{
    fn base_to_custom(ev:NetEvent) -> NetEvent {
        ev
    }
    fn custom_to_base(ev: NetEvent) -> Option<NetEvent> {
        Some(ev)
    }
}

/// The BaseNetEvent trait is used to convert from a user-made event enum to the predefined amethyst base event enum and the inverse.
pub trait BaseNetEvent<T>{
    /// Converts a base event to a user event
    fn base_to_custom(ev:NetEvent)->T;
    /// Converts a user event to a base event (if applicable)
    fn custom_to_base(ev:T)->Option<NetEvent>;
}*/

///Carries the source of the event. Useful for debugging, security checks, gameplay logic, etc...
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetOwnedEvent<T> where T:Event{
    /// The event
    pub event: T,
    /// The source of this event
    pub owner: NetConnection,
}

