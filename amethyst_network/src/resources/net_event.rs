use shrev::Event;

use resources::connection::*;

///The basic network events shipped with amethyst
///You can add more event by making a new enum and having this one has a variant of yours.
//TODO: Add CreateEntity,RemoveEntity,UpdateEntity
//TODO: Example of switching the NetEvent set in the Network Systems
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum NetEvent{
    Connect,
    Connected,
    ConnectionRefused{reason:String},
    Disconnect{reason:String},
    Disconnected{reason:String},
}

//impl Event for NetEvent{}

///Carries the source of the event. Useful for debugging, security checks, gameplay logic, etc...
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetOwnedEvent<T> where T:Event{
    pub event:T,
    pub owner:NetConnection,
}

//impl<T> Event for NetOwnedEvent<T>{}