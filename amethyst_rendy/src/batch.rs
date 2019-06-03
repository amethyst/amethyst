use crate::util::TapCountIter;
use derivative::Derivative;
use smallvec::{smallvec, SmallVec};
use std::{
    collections::hash_map::Entry,
    iter::{Extend, FromIterator},
    ops::Range,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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
        #[cfg(feature = "profiler")]
        profile_scope!("for_each_group");

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
        self.map.retain(|_, b| !b.is_empty());
    }

    pub fn insert(&mut self, pk: PK, sk: SK, data: impl IntoIterator<Item = C::Item>) {
        #[cfg(feature = "profiler")]
        profile_scope!("twolevel_insert");

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

    pub fn data(&self) -> impl Iterator<Item = &C> {
        self.map
            .iter()
            .flat_map(|(_, batch)| batch.iter().map(|data| &data.1))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PK, impl Iterator<Item = &(SK, C)>)> {
        self.map.iter().map(|(pk, batch)| (pk, batch.iter()))
    }

    pub fn count(&self) -> usize {
        self.data_count
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OrderedTwoLevelBatch<PK, SK, D>
where
    PK: PartialEq,
    SK: PartialEq,
{
    old_pk_list: Vec<(PK, u32)>,
    old_sk_list: Vec<(SK, Range<u32>)>,
    pk_list: Vec<(PK, u32)>,
    sk_list: Vec<(SK, Range<u32>)>,
    data_list: Vec<D>,
}

impl<PK, SK, D> OrderedTwoLevelBatch<PK, SK, D>
where
    PK: PartialEq,
    SK: PartialEq,
{
    pub fn swap_clear(&mut self) {
        std::mem::swap(&mut self.old_pk_list, &mut self.pk_list);
        std::mem::swap(&mut self.old_sk_list, &mut self.sk_list);
        self.pk_list.clear();
        self.sk_list.clear();
        self.data_list.clear();
    }

    pub fn insert(&mut self, pk: PK, sk: SK, data: impl IntoIterator<Item = D>) {
        #[cfg(feature = "profiler")]
        profile_scope!("ordered_twolevel_insert");

        let start = self.data_list.len() as u32;
        self.data_list.extend(data);
        let end = self.data_list.len() as u32;

        match (self.pk_list.last_mut(), self.sk_list.last_mut()) {
            (Some((last_pk, _)), Some((last_sk, last_sk_range)))
                if last_pk == &pk && last_sk == &sk =>
            {
                last_sk_range.end = end;
            }
            (Some((last_pk, last_pk_len)), _) if last_pk == &pk => {
                *last_pk_len += 1;
                self.sk_list.push((sk, start..end));
            }
            _ => {
                self.pk_list.push((pk, 1));
                self.sk_list.push((sk, start..end));
            }
        }
    }

    pub fn data(&self) -> &Vec<D> {
        &self.data_list
    }

    /// Iterator that returns primary keys and all inner submitted batches
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a PK, &[(SK, Range<u32>)])> {
        let mut pk_offset = 0;
        self.pk_list.iter().map(move |(pk, pk_len)| {
            let range = pk_offset..pk_offset + *pk_len as usize;
            pk_offset += *pk_len as usize;
            (pk, &self.sk_list[range])
        })
    }

    pub fn changed(&self) -> bool {
        self.pk_list != self.old_pk_list || self.sk_list != self.old_sk_list
    }

    pub fn count(&self) -> usize {
        self.data_list.len()
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OneLevelBatch<PK, D>
where
    PK: Eq + std::hash::Hash,
{
    map: fnv::FnvHashMap<PK, Vec<D>>,
    data_count: usize,
}

impl<PK, D> OneLevelBatch<PK, D>
where
    PK: Eq + std::hash::Hash,
{
    pub fn clear_inner(&mut self) {
        self.data_count = 0;
        for (_, data) in self.map.iter_mut() {
            data.clear();
        }
    }

    pub fn prune(&mut self) {
        self.map.retain(|_, b| !b.is_empty());
    }

    pub fn insert(&mut self, pk: PK, data: impl IntoIterator<Item = D>) {
        #[cfg(feature = "profiler")]
        profile_scope!("onelevel_insert");

        let instance_data = data.into_iter();

        match self.map.entry(pk) {
            Entry::Occupied(mut e) => {
                let vec = e.get_mut();
                let old_len = vec.len();
                vec.extend(instance_data);
                self.data_count += vec.len() - old_len;
            }
            Entry::Vacant(e) => {
                let collected = instance_data.collect::<Vec<_>>();
                self.data_count += collected.len();
                e.insert(collected);
            }
        }
    }

    pub fn data<'a>(&'a self) -> impl Iterator<Item = &'a Vec<D>> {
        self.map.values()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a PK, Range<u32>)> {
        let mut offset = 0;
        self.map.iter().map(move |(pk, data)| {
            let range = offset..offset + data.len() as u32;
            offset = range.end;
            (pk, range)
        })
    }

    pub fn count(&self) -> usize {
        self.data_count
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OrderedOneLevelBatch<PK, D>
where
    PK: PartialEq,
{
    old_keys: Vec<(PK, u32)>,
    keys_list: Vec<(PK, u32)>,
    data_list: Vec<D>,
}

impl<PK, D> OrderedOneLevelBatch<PK, D>
where
    PK: PartialEq,
{
    pub fn swap_clear(&mut self) {
        std::mem::swap(&mut self.old_keys, &mut self.keys_list);
        self.keys_list.clear();
        self.data_list.clear();
    }

    pub fn insert(&mut self, pk: PK, data: impl IntoIterator<Item = D>) {
        #[cfg(feature = "profiler")]
        profile_scope!("ordered_onelevel_insert");

        let start = self.data_list.len() as u32;
        self.data_list.extend(data);
        let added_len = self.data_list.len() as u32 - start;

        match self.keys_list.last_mut() {
            Some((last_pk, last_len)) if last_pk == &pk => {
                *last_len += added_len;
            }
            _ => {
                self.keys_list.push((pk, added_len));
            }
        }
    }

    pub fn data(&self) -> &Vec<D> {
        &self.data_list
    }

    /// Iterator that returns primary keys and lengths of submitted batch
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a PK, Range<u32>)> {
        let mut offset = 0;
        self.keys_list.iter().map(move |(pk, size)| {
            let range = offset..offset + *size;
            offset = range.end;
            (pk, range)
        })
    }

    pub fn changed(&self) -> bool {
        self.keys_list != self.old_keys
    }

    pub fn count(&self) -> usize {
        self.data_list.len()
    }
}
