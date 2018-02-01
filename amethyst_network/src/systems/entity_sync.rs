
pub struct EntitySyncSystem<T> {
    pub net_event_reader: ReaderId,
}

impl<'a,T> System<'a> for EntitySyncSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{
    type SystemData = (
        Entities,
        Fetch<'a, EventChannel<NetOwnedEvent<T>>>,
    );
    fn run(&mut self, mut events: Self::SystemData) {
        //let ev = readeventblablabla;
        match T::custom_to_base(ev){
            Some(NetEvent::WriteComponent(netsync))=>{
                // TODO: Needs filtering to exclude !owner
                // TODO: Someone with more specs knowledge than me should do this part.
            },
            None=>{},
        }
    }
}