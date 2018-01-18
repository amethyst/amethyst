
use specs::{Component, DenseVecStorage};

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight(pub [f32; 3]);

#[derive(Clone, Copy, Debug)]
pub struct PointLight(pub f32);

#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
}

impl Component for Light {
    type Storage = DenseVecStorage<Light>;
}
