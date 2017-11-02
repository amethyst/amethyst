use specs::{Fetch, SystemData, World};

use asset::Asset;
use storage::{AssetStorage, DescStorage};

pub trait FetchStorages<S> {
    fn fetch(self) -> S;
}

impl<'a, A, B> FetchStorages<(DescStorage<A>, DescStorage<B>)> for (DescStorage<A>, DescStorage<B>)
where
    A: Asset,
    B: Asset,
{
    fn fetch(self) -> (DescStorage<A>, DescStorage<B>) {
        self
    }
}


impl<'a, A, B> FetchStorages<(DescStorage<A>, DescStorage<B>)>
    for (Fetch<'a, AssetStorage<A>>, Fetch<'a, AssetStorage<B>>)
where
    A: Asset,
    B: Asset,
{
    fn fetch(self) -> (DescStorage<A>, DescStorage<B>) {
        (self.0.desc_storage(), self.1.desc_storage())
    }
}

impl<'a, A, B> FetchStorages<(DescStorage<A>, DescStorage<B>)> for &'a World
where
    A: Asset,
    B: Asset,
{
    fn fetch(self) -> (DescStorage<A>, DescStorage<B>) {
        let d: (Fetch<'a, AssetStorage<A>>, Fetch<'a, AssetStorage<B>>) =
            SystemData::fetch(&self.res, 0);

        d.fetch()
    }
}
