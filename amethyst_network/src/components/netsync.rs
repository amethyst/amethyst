use specs::Component;
use std::marker::PhantomData;

/// The component that marks an entity as synchronized to a remote network host. (Unfinished)
pub struct NetSync<T>
where
    T: Component,
{
    item: PhantomData<T>,
}
