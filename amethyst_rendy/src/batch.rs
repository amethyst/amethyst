use crate::util::TapCountIter;
use derivative::Derivative;
use smallvec::{smallvec, SmallVec};
use std::{
    collections::hash_map::Entry,
    iter::{Extend, FromIterator},
};

pub trait GroupIterator<K, V>
where
    Self: Iterator<Item = (K, V)> + Sized,
    K: PartialEq,
{
    /// Perform grouping. Evaluates passed closure on every next
    /// countiguous list of data with same group identifier.
    fn for_each_group<F>(self, on_group: F)
    where
        F: FnMut(K, &mut Vec<V>);
}

// This would be an iterator adaptor if `Item` type would allow a borrow on iterator itself.
// FIXME: Implement once `StreamingIterator` is a thing.
impl<K, V, I> GroupIterator<K, V> for I
where
    K: PartialEq,
    I: Iterator<Item = (K, V)>,
{
    fn for_each_group<F>(self, mut on_group: F)
    where
        F: FnMut(K, &mut Vec<V>),
    {
        let mut block: Option<(K, Vec<V>)> = None;

        for (next_group_id, value) in self {
            match &mut block {
                slot @ None => {
                    let mut group_buffer = Vec::with_capacity(64);
                    group_buffer.push(value);
                    slot.replace((next_group_id, group_buffer));
                }
                Some((group_id, group_buffer)) if group_id == &next_group_id => {
                    group_buffer.push(value);
                }
                Some((group_id, ref mut group_buffer)) => {
                    let submitted_group_id = std::mem::replace(group_id, next_group_id);
                    on_group(submitted_group_id, group_buffer);
                    group_buffer.clear();
                    group_buffer.push(value);
                }
            }
        }

        if let Some((group_id, mut group_buffer)) = block.take() {
            on_group(group_id, &mut group_buffer);
        }
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct TwoLevelBatch<PK, SK, C>
where
    PK: Eq + std::hash::Hash,
{
    map: fnv::FnvHashMap<PK, SmallVec<[(SK, C); 1]>>,
    data_count: usize,
}

impl<PK, SK, C> TwoLevelBatch<PK, SK, C>
where
    PK: Eq + std::hash::Hash,
    SK: PartialEq,
    C: IntoIterator,
    C: FromIterator<<C as IntoIterator>::Item>,
    C: Extend<<C as IntoIterator>::Item>,
{
    pub fn clear_inner(&mut self) {
        self.data_count = 0;
        for (_, data) in self.map.iter_mut() {
            data.clear();
        }
    }

    pub fn prune(&mut self) {
        self.map.retain(|_, b| b.len() > 0);
    }

    pub fn insert(&mut self, pk: PK, sk: SK, data: impl IntoIterator<Item = C::Item>) {
        let instance_data = data.into_iter().tap_count(&mut self.data_count);

        match self.map.entry(pk) {
            Entry::Occupied(mut e) => {
                let e = e.get_mut();
                // scan for the same key to try to combine batches.
                // Scanning limited slots to limit complexity.
                if let Some(batch) = e.iter_mut().take(8).find(|(k, _)| k == &sk) {
                    batch.1.extend(instance_data);
                } else {
                    e.push((sk, instance_data.collect()));
                }
            }
            Entry::Vacant(e) => {
                e.insert(smallvec![(sk, instance_data.collect())]);
            }
        }
    }

    pub fn data<'a>(&'a self) -> impl Iterator<Item = &'a C> {
        self.map
            .iter()
            .flat_map(|(_, batch)| batch.iter().map(|data| &data.1))
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a PK, impl Iterator<Item = &'a (SK, C)>)> {
        self.map.iter().map(|(pk, batch)| (pk, batch.iter()))
    }

    pub fn count(&self) -> usize {
        self.data_count
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OrderedTwoLevelBatch<PK, SK, C> {
    list: Vec<(PK, SK, C)>,
    data_count: usize,
}

impl<PK, SK, C> OrderedTwoLevelBatch<PK, SK, C>
where
    PK: Eq + Copy,
    SK: PartialEq,
    C: IntoIterator,
    C: FromIterator<<C as IntoIterator>::Item>,
    C: Extend<<C as IntoIterator>::Item>,
{
    pub fn clear(&mut self) {
        self.list.clear();
        self.data_count = 0;
    }

    pub fn insert(&mut self, pk: PK, sk: SK, data: impl IntoIterator<Item = C::Item>) {
        let instance_data = data.into_iter().tap_count(&mut self.data_count);

        match self.list.last_mut() {
            Some((last_pk, last_sk, c)) if last_pk == &pk && last_sk == &sk => {
                c.extend(instance_data);
            }
            _ => self.list.push((pk, sk, instance_data.collect())),
        }
    }

    pub fn data<'a>(&'a self) -> impl Iterator<Item = &'a C> {
        self.list.iter().map(|(_, _, data)| data)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (PK, impl Iterator<Item = (&'a SK, &'a C)>)> {
        (0..).scan(0, move |start_idx, _| {
            self.list.get(*start_idx).map(|(pk, _, _)| {
                let size = self.list[*start_idx..]
                    .iter()
                    .take_while(|e| &e.0 == pk)
                    .count();
                let range = *start_idx..*start_idx + size;
                *start_idx += size;
                (*pk, self.list[range].iter().map(|(_, sk, c)| (sk, c)))
            })
        })
    }

    pub fn count(&self) -> usize {
        self.data_count
    }
}
