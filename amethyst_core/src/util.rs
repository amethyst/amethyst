use hibitset::BitSetLike;
use specs::{Component, Entity, storage::UnprotectedStorage};

pub trait Cache<T> {
    fn on_get(&self, id: u32, val: &T);
    fn on_update(&mut self, id: u32, val: &T);
    fn on_remove(&mut self, id: u32, val: T) -> T;
}

#[derive(Debug, Default)]
pub struct CachedStorage<C, I, T> {
    pub cache: C,
    pub inner: I,
}

impl<C, I, T> UnprotectedStorage for CachedStorage<C, I, T> where C: Cache<T::Com>, I: UnprotectedStorage<>, T: Component {
    unsafe fn clean<B>(&mut self, has: B) where
        B: BitSetLike {
        self.inner.clean(has);
    }

    unsafe fn get(&self, id: u32) -> &T {
        let val = self.inner.get(id);
        self.cache.on_get(id, val);

        val
    }

    unsafe fn get_mut(&mut self, id: u32) -> &mut T {
        let val = self.inner.get(id);
        self.cache.on_update(id, val);

        val
    }

    unsafe fn insert(&mut self, id: u32, value: T) {
        self.cache.on_update(id, &value);
        self.inner.insert(id, value);
    }

    unsafe fn remove(&mut self, id: u32) -> T {
        let val = self.inner.remove(id);

        self.cache.on_remove(id, val)
    }
}
