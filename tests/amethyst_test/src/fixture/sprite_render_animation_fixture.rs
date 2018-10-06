use amethyst::{
    animation::{
        Animation, InterpolationFunction, Sampler, SpriteRenderChannel, SpriteRenderPrimitive,
    },
    assets::{AssetStorage, Handle, Loader},
    ecs::prelude::*,
    renderer::SpriteRender,
};

use EffectReturn;

/// Fixture to test sprite render animation loading.
#[derive(Debug)]
pub struct SpriteRenderAnimationFixture;

impl SpriteRenderAnimationFixture {
    /// Loads a sprite render animation into the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load the sprite render animation into.
    pub fn effect(world: &mut World) {
        // Load the animation.
        let animation_handle = {
            let sprite_sheet_sampler = Sampler {
                input: vec![0.0],
                output: vec![SpriteRenderPrimitive::SpriteSheet(0)],
                function: InterpolationFunction::Step,
            };
            let sprite_index_sampler = Sampler {
                input: vec![0.0],
                output: vec![SpriteRenderPrimitive::SpriteIndex(0)],
                function: InterpolationFunction::Step,
            };

            let loader = world.read_resource::<Loader>();
            let sprite_sheet_animation_handle =
                loader.load_from_data(sprite_sheet_sampler, (), &world.read_resource());
            let sprite_index_animation_handle =
                loader.load_from_data(sprite_index_sampler, (), &world.read_resource());

            let animation = Animation::<SpriteRender> {
                nodes: vec![
                    (
                        0,
                        SpriteRenderChannel::SpriteSheet,
                        sprite_sheet_animation_handle,
                    ),
                    (
                        0,
                        SpriteRenderChannel::SpriteIndex,
                        sprite_index_animation_handle,
                    ),
                ],
            };

            loader.load_from_data::<Animation<SpriteRender>, ()>(
                animation,
                (),
                &world.read_resource(),
            )
        };
        world.add_resource(EffectReturn(animation_handle));
    }

    /// Asserts that the sprite render animation is present in the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` that the sprite render animation is loaded in.
    pub fn assertion(world: &mut World) {
        // Read the animation.
        let animation_handle = &world
            .read_resource::<EffectReturn<Handle<Animation<SpriteRender>>>>()
            .0;

        let store = world.read_resource::<AssetStorage<Animation<SpriteRender>>>();
        assert!(store.get(animation_handle).is_some());
    }
}
