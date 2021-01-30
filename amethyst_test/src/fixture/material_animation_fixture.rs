use amethyst::{
    animation::{Animation, InterpolationFunction, MaterialChannel, MaterialPrimitive, Sampler},
    assets::{AssetStorage, DefaultLoader, Handle, Loader},
    ecs::*,
    renderer::{loaders::load_from_srgba, palette::Srgba, Material},
};

use crate::EffectReturn;

/// Fixture to test material animation loading.
#[derive(Debug)]
pub struct MaterialAnimationFixture;

impl MaterialAnimationFixture {
    /// Loads a material animation into the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load the material animation into.
    pub fn effect(world: &mut World) {
        // Load the animation.
        let animation_handle = {
            let loader = data.resources.get::<DefaultLoader>().unwrap();
            let tex_handle = loader.load_from_data(
                load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 0.5)).into(),
                (),
                &world.read_resource(),
            );

            let texture_sampler = Sampler {
                input: vec![0.0],
                output: vec![MaterialPrimitive::Texture(tex_handle)],
                function: InterpolationFunction::Step,
            };
            let sprite_offset_sampler = Sampler {
                input: vec![0.0],
                output: vec![MaterialPrimitive::Offset((0.0, 1.0), (1.0, 0.0))],
                function: InterpolationFunction::Step,
            };

            let texture_animation_handle =
                loader.load_from_data(texture_sampler, (), &world.read_resource());
            let sampler_animation_handle =
                loader.load_from_data(sprite_offset_sampler, (), &world.read_resource());

            let animation = Animation::<Material> {
                nodes: vec![
                    (0, MaterialChannel::AlbedoTexture, texture_animation_handle),
                    (0, MaterialChannel::UvOffset, sampler_animation_handle),
                ],
            };

            loader.load_from_data::<Animation<Material>, ()>(animation, (), &world.read_resource())
        };
        world.insert(EffectReturn(animation_handle));
    }

    /// Asserts that the material animation is present in the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` that the material animation is loaded in.
    pub fn assertion(world: &mut World) {
        // Read the animation.
        let animation_handle = &world
            .read_resource::<EffectReturn<Handle<Animation<Material>>>>()
            .0;

        let store = world.read_resource::<AssetStorage<Animation<Material>>>();
        assert!(store.get(animation_handle).is_some());
    }
}
