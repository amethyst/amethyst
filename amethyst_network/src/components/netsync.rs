use specs::Component;
use std::marker::PhantomData;

pub struct NetSync<T> where T: Component{
    item:PhantomData<T>
}