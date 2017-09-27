use std::error::Error;
use std::marker::PhantomData;

use serde::ser::{self, Serialize, SerializeSeq, Serializer};

use shred::{ResourceId, Resources, SystemData};
use specs::{Entities, FetchMut, Join, ReadStorage, WriteStorage};

use super::{Components, EntityData, Storages};
use super::marker::{Marker, MarkerAllocator};


/// Serialize components in tuple `T` of entities marked by `M`
/// with serializer `S`
/// All entities referenced in serialized get marked and serialized recursively
/// For serializing without such recursion see `serialize` function.
pub fn serialize_recursive<'a, M, E, T, S>(
    entities: &Entities<'a>,
    storages: &<T as Storages<'a>>::ReadStorages,
    markers: &mut WriteStorage<'a, M>,
    allocator: &mut FetchMut<'a, M::Allocator>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
    S: Serializer,
{
    let mut serseq = serializer.serialize_seq(None)?;
    let mut to_serialize = (&**entities, &*markers)
        .join()
        .map(|(e, m)| (e, *m))
        .collect::<Vec<_>>();
    loop {
        if to_serialize.is_empty() {
            break;
        }
        let mut add = vec![];
        {
            let mut ids = |entity| -> Option<M::Identifier> {
                match markers.get(entity).cloned() {
                    Some(marker) => Some(marker.id()),
                    None => {
                        let marker = allocator.mark(entity, markers);
                        add.push((entity, marker));
                        Some(marker.id())
                    }
                }
            };
            for (entity, marker) in to_serialize.into_iter() {
                serseq.serialize_element(&EntityData::<M, E, T> {
                    marker: marker,
                    components: T::save(entity, storages, &mut ids).map_err(ser::Error::custom)?,
                })?;
            }
        }
        to_serialize = add;
    }
    serseq.end()
}



/// Serialize components in tuple `T` of entities marked by `M`
/// with serializer `S`
/// Doesn't recursively mark referenced entities.
/// Closure passed in `SerializableComponent::save` returns `None` for unmarked `Entity`
/// In this case `SerializableComponent::save` may perform workaround
/// (forget about `Entity`) or fail
/// For recursive marking see `serialize_recursive`
pub fn serialize<'a, M, E, T, S>(
    entities: &Entities<'a>,
    storages: &<T as Storages<'a>>::ReadStorages,
    markers: &ReadStorage<'a, M>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
    S: Serializer,
{
    let mut serseq = serializer.serialize_seq(None)?;
    let ids = |entity| -> Option<M::Identifier> { markers.get(entity).map(Marker::id) };
    for (entity, marker) in (&**entities, &*markers).join() {
        serseq.serialize_element(&EntityData::<M, E, T> {
            marker: *marker,
            components: T::save(entity, storages, &ids).map_err(ser::Error::custom)?,
        })?;
    }
    serseq.end()
}

/// This type implements `Serialize` so that it may be used in generic environment
/// where `Serialize` implementer is expected
/// It may be constructed manually (TODO: Add `new` function)
/// Or fetched by `System` as `SystemData`
/// Serializes components in tuple `T` with marker `M`
pub struct WorldSerialize<'a, M: Marker, E: Error, T: Components<M::Identifier, E>> {
    entities: Entities<'a>,
    storages: <T as Storages<'a>>::ReadStorages,
    markers: ReadStorage<'a, M>,
    pd: PhantomData<E>,
}

impl<'a, M, E, T> WorldSerialize<'a, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    /// Remove all marked entities
    /// Use it if you want to delete entities that was just serialized
    pub fn remove_serialized(&mut self) {
        for (entity, _) in (&*self.entities, &self.markers.check()).join() {
            self.entities.delete(entity);
        }
    }
}


impl<'a, M, E, T> SystemData<'a> for WorldSerialize<'a, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    fn fetch(res: &'a Resources, id: usize) -> Self {
        WorldSerialize {
            entities: Entities::<'a>::fetch(res, id),
            storages: <T as Storages<'a>>::ReadStorages::fetch(res, id),
            markers: ReadStorage::<'a, M>::fetch(res, id),
            pd: PhantomData,
        }
    }
    fn reads(id: usize) -> Vec<ResourceId> {
        let mut reads = Entities::<'a>::reads(id);
        reads.extend(<T as Storages<'a>>::ReadStorages::reads(id));
        reads.extend(ReadStorage::<'a, M>::reads(id));
        reads
    }
    fn writes(_id: usize) -> Vec<ResourceId> {
        Vec::new()
    }
}

impl<'a, M, E, T> Serialize for WorldSerialize<'a, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize::<M, E, T, S>(&self.entities, &self.storages, &self.markers, serializer)
    }
}
