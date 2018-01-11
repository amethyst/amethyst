use shrev::Event;

use resources::connection::*;

///The basic network events shipped with amethyst
///You can add more event by making a new enum and having this one has a variant of yours.
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
///             SampleUserEvents::Base(e)=>Some(e),
///             _=>None,
///         }
///     }
/// }
//TODO: Add CreateEntity,RemoveEntity,UpdateEntity
//TODO: Example of switching the NetEvent set in the Network Systems
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum NetEvent{
    Connect,
    Connected,
    ConnectionRefused{reason:String},
    Disconnect{reason:String},
    Disconnected{reason:String},
}

impl BaseNetEvent<NetEvent> for NetEvent{
    fn base_to_custom(ev:NetEvent) -> NetEvent {
        ev
    }
    fn custom_to_base(ev: NetEvent) -> Option<NetEvent> {
        Some(ev)
    }
}

pub trait BaseNetEvent<T>{
    fn base_to_custom(ev:NetEvent)->T;
    fn custom_to_base(ev:T)->Option<NetEvent>;
}

///Carries the source of the event. Useful for debugging, security checks, gameplay logic, etc...
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetOwnedEvent<T> where T:Event{
    pub event:T,
    pub owner:NetConnection,
}

