
use std::error::Error;
use std::fmt::{self, Formatter};
use std::marker::PhantomData;

use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};

use shred::{ResourceId, Resources, SystemData};
use specs::{Entities, FetchMut, WriteStorage};

use super::{Components, EntityData, Storages};
use super::marker::{Marker, MarkerAllocator};


/// Wrapper for `Entity` and tuple of `WriteStorage`s that implements `serde::Deserialize`
struct DeserializeEntity<'a, 'b: 'a, M: Marker, E: Error, T: Components<M::Identifier, E>> {
    entities: &'a Entities<'b>,
    storages: &'a mut <T as Storages<'b>>::WriteStorages,
    markers: &'a mut WriteStorage<'b, M>,
    allocator: &'a mut FetchMut<'b, M::Allocator>,
    pd: PhantomData<(M, E, T)>,
}

impl<'de, 'a, 'b: 'a, M, E, T> DeserializeSeed<'de> for DeserializeEntity<'a, 'b, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        let DeserializeEntity {
            entities,
            storages,
            markers,
            allocator,
            ..
        } = self;
        let data = EntityData::<M, E, T>::deserialize(deserializer)?;
        let entity = allocator.marked(data.marker.id(), entities, markers);
        markers
            .get_mut(entity)
            .ok_or("Allocator is broken")
            .map_err(de::Error::custom)?
            .update(data.marker);
        let ids = |marker: M::Identifier| Some(allocator.marked(marker, entities, markers));
        T::load(entity, data.components, storages, ids).map_err(de::Error::custom)?;
        Ok(())
    }
}


/// Wrapper for `Entities` and tuple of `WriteStorage`s that implements `serde::Deserialize`
struct VisitEntities<'a, 'b: 'a, M: Marker, E: Error, T: Components<M::Identifier, E>> {
    entities: &'a Entities<'b>,
    storages: &'a mut <T as Storages<'b>>::WriteStorages,
    markers: &'a mut WriteStorage<'b, M>,
    allocator: &'a mut FetchMut<'b, M::Allocator>,
    pd: PhantomData<(M, E, T)>,
}

impl<'de, 'a, 'b: 'a, M, E, T> Visitor<'de> for VisitEntities<'a, 'b, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Sequence of serialized entities")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(()) = seq.next_element_seed(DeserializeEntity {
            entities: self.entities,
            storages: self.storages,
            markers: self.markers,
            allocator: self.allocator,
            pd: self.pd,
        })? {}

        Ok(())
    }
}


/// Deserialize entities
pub fn deserialize<'a, 'de, D, M, E, T>(
    entities: &Entities<'a>,
    storages: &mut <T as Storages<'a>>::WriteStorages,
    markers: &mut WriteStorage<'a, M>,
    allocator: &mut FetchMut<'a, M::Allocator>,
    deserializer: D,
) -> Result<(), D::Error>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(VisitEntities::<M, E, T> {
        entities: entities,
        storages: storages,
        markers: markers,
        allocator: allocator,
        pd: PhantomData,
    })
}


/// `DeerializeSeed` implementation for `World`
//#[derive(SystemData)]
pub struct WorldDeserialize<'a, M: Marker, E: Error, T: Components<M::Identifier, E>> {
    entities: Entities<'a>,
    storages: <T as Storages<'a>>::WriteStorages,
    markers: WriteStorage<'a, M>,
    allocator: FetchMut<'a, M::Allocator>,
    pd: PhantomData<E>,
}

impl<'a, M, E, T> SystemData<'a> for WorldDeserialize<'a, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    fn fetch(res: &'a Resources, id: usize) -> Self {
        WorldDeserialize {
            entities: Entities::<'a>::fetch(res, id),
            storages: <T as Storages<'a>>::WriteStorages::fetch(res, id),
            markers: WriteStorage::<'a, M>::fetch(res, id),
            allocator: FetchMut::<'a, M::Allocator>::fetch(res, id),
            pd: PhantomData,
        }
    }
    fn reads(id: usize) -> Vec<ResourceId> {
        let mut reads = Entities::<'a>::reads(id);
        reads.extend(<T as Storages<'a>>::WriteStorages::reads(id));
        reads.extend(WriteStorage::<'a, M>::reads(id));
        reads.extend(FetchMut::<'a, M::Allocator>::reads(id));
        reads
    }
    fn writes(id: usize) -> Vec<ResourceId> {
        let mut writes = Entities::<'a>::writes(id);
        writes.extend(<T as Storages<'a>>::WriteStorages::writes(id));
        writes.extend(WriteStorage::<'a, M>::writes(id));
        writes.extend(FetchMut::<'a, M::Allocator>::writes(id));
        writes
    }
}

impl<'de, 'a, M, E, T> DeserializeSeed<'de> for WorldDeserialize<'a, M, E, T>
where
    M: Marker,
    E: Error,
    T: Components<M::Identifier, E>,
{
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize::<D, M, E, T>(
            &mut self.entities,
            &mut self.storages,
            &mut self.markers,
            &mut self.allocator,
            deserializer,
        )
    }
}
