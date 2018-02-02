use specs::*;
use specs::saveload::*;
use specs::error::NoError;
use shrev::*;
use resources::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct EntitySyncSystem<T> {
    pub net_event_reader: Option<ReaderId<U64Marker>>,
}

impl<'a,T> System<'a> for EntitySyncSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    type SystemData = (
        WorldSerialize<'a,U64Marker,NoError,(TestComp)>,
        Fetch<'a, EventChannel<NetOwnedEvent<NetEvent<T>>>>,
    );
    fn run(&mut self, (world_ser,events): Self::SystemData) {
        if self.net_event_reader.is_none(){
            self.net_event_reader = Some(events.register_reader());
        }

        for ev in events.read(self.net_event_reader.as_mut().unwrap()){
            info!("RECEIVED NET EVENT");
        }

        /*//let ev = readeventblablabla;
        match T::custom_to_base(ev){
            Some(NetEvent::WriteComponent(netsync))=>{
                // TODO: Needs filtering to exclude !owner
                // TODO: Someone with more specs knowledge than me should do this part.
            },
            None=>{},
        }*/
    }
}