use specs::Component;
use std::marker::PhantomData;

pub struct NetSync<T> where T: Component{
    item:PhantomData<T>
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TestSer {
    pub x: f32,
    pub y: f32,
}