
use specs::{Component, DenseVecStorage};

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight(pub [f32; 3]);

#[derive(Clone, Copy, Debug)]
pub struct PointLight(pub [f32; 3]);

impl<T> From<T> for PointLight
where
    T: Into<[f32; 3]>,
{
    fn from(value: T) -> PointLight {
        PointLight(value.into())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
}

impl From<PointLight> for Light {
    fn from(point: PointLight) -> Light {
        Light::Point(point)
    }
}

impl Component for Light {
    type Storage = DenseVecStorage<Light>;
}
