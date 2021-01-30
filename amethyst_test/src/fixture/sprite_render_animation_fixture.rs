use amethyst::{
    animation::{
        Animation, InterpolationFunction, Sampler, SpriteRenderChannel, SpriteRenderPrimitive,
    },
    assets::{AssetStorage, DefaultLoader, Handle, Loader},
    ecs::*,
    renderer::{
        loaders::load_from_srgba, palette::Srgba, Sprite, SpriteRender, SpriteSheet, Texture,
    },
};

use crate::EffectReturn;

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
            // This invocation sequence of read_resource / write_resource is to satisfy the borrow
            // checker, since we don't have NLL yet.
            let tex_handle = world.read_resource::<DefaultLoader>().load_from_data(
                load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 0.5)).into(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );

            let loader = data.resources.get::<DefaultLoader>().unwrap();
            let sprite_sheet_handle =
                loader.load_from_data(Self::sprite_sheet(tex_handle), (), &world.read_resource());
            let sprite_sheet_sampler = Sampler {
                input: vec![0.0],
                output: vec![SpriteRenderPrimitive::SpriteSheet(sprite_sheet_handle)],
                function: InterpolationFunction::Step,
            };
            let sprite_index_sampler = Sampler {
                input: vec![0.0],
                output: vec![SpriteRenderPrimitive::SpriteIndex(0)],
                function: InterpolationFunction::Step,
            };

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
        world.insert(EffectReturn(animation_handle));
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

    fn sprite_sheet(texture: Handle<Texture>) -> SpriteSheet {
        SpriteSheet {
            texture,
            sprites: vec![Sprite {
                width: 10.0,
                height: 10.0,
                offsets: [5.; 2],
                tex_coords: [0.0, 1.0, 0.0, 1.0].into(),
            }],
        }
    }
}
