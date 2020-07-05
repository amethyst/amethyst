use amethyst_assets::build_asset_processor_system;

pub use self::{
    control::{AnimationControlSystem},
    sampling::build_sampler_interpolation_system,
};
use crate::resources::{Animation, Sampler};
use amethyst_core::ecs::prelude::*;

mod control;
mod sampling;

/// Asset storage processor for `Sampler`
// pub type SamplerProcessor<S> = Processor<Sampler<S>>;
pub fn build_sampler_processor<S>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    build_asset_processor_system::<Sampler<S>>(world, resources)
} 

/// Asset storage processor for `Animation`
// pub type AnimationProcessor<T> = Processor<Animation<T>>;

pub fn build_animation_processor<S>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    build_asset_processor_system::<Animation<S>>(world, resources)
} 