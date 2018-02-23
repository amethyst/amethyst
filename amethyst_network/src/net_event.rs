//! The network events that are passed from machine to machine, and within the ECS event handling system.
//! NetEvent are passed through the network
//! NetOwnedEvent are passed through the ECS, and contains the event's source (remote connection, usually)

use shrev::Event;
use serde::Serialize;
use serde::de::DeserializeOwned;
use specs::{VecStorage,Component};
use uuid::Uuid;
use std::net::SocketAddr;

/// The basic network events shipped with amethyst.
// TODO: Add CreateEntity,RemoveEntity,UpdateEntity
// TODO: Example of switching the NetEvent set in the Network Systems
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum NetEvent<T>{
    /// Ask to connect to the server
    Connect,
    /// Reply to the client that the connection has been accepted
    Connected{
        server_uuid: Uuid,
    },
    /// Reply to the client that the connection has been refused
    ConnectionRefused{
        /// The reason of the refusal
        reason:String
    },
    /// Tell the server that the client is disconnecting
    Disconnect{
        /// The reason of the disconnection
        reason:String,
    },
    /// Notify the clients(including the one being disconnected) that a client has been disconnected from the server
    Disconnected{
        /// The reason of the disconnection
        reason:String,
    },
    AddComponent{
        compData:String,
    },
    /// A user-defined enum containing more network event types.
    Custom(T),
}

impl<T> NetEvent<T> {
    /// Tries to convert a NetEvent to a custom event enum type.
    pub fn custom(&self) -> Option<&T> {
        if let &NetEvent::Custom(ref t) = self {
            Some(&t)
        } else {
            None
        }
    }
}


// for later testing of component sync
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestComp{
    data:String,
}

impl Component for TestComp{
    type Storage = VecStorage<TestComp>;
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
pub struct NetSourcedEvent<T>{
    /// The event.
    pub event: NetEvent<T>,
    /// The source of this event.
    /// Might be none if the client is connecting.
    pub uuid: Option<Uuid>,
    /// The socket which sent this event.
    pub socket: SocketAddr,
}

