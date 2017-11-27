//use specs::{Fetch, SystemData, World};
//
//use asset::Asset;
//use storage::{AssetStorage, DescStorage};
//
//pub trait FetchStorages {
//    type Fetched;
//
//    fn fetch(self) -> Self::Fetched;
//}
//
//impl<A> FetchStorages for DescStorage<A> {
//    type Fetched = Self;
//
//    fn fetch(self) -> Self::Fetched {
//        self
//    }
//}
//
//impl<'a, A> FetchStorages for Fetch<'a, AssetStorage<A>> {
//    type Fetched = DescStorage<A>;
//
//    fn fetch(self) -> Self::Fetched {
//        self.desc_storage()
//    }
//}
//
//impl<A, B> FetchStorages for (A, B)
//where
//    A: FetchStorages,
//    B: FetchStorages,
//{
//    type Fetched = (A::Fetched, B::Fetched);
//
//    fn fetch(self) -> Self::Fetched {
//        (self.0.fetch(), self.1.fetch())
//    }
//}
