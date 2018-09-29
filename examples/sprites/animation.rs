use amethyst::{
    assets::{Handle, Loader},
    prelude::*,
    renderer::SpriteRender,
};
use amethyst_animation::{
    Animation, InterpolationFunction, Sampler, SpriteRenderChannel, SpriteRenderPrimitive,
};

pub fn grey_bat(world: &mut World, sprite_sheet_id: u64) -> Handle<Animation<SpriteRender>> {
    let sprite_indicies = [5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 4]
        .into_iter()
        .map(|&n| SpriteRenderPrimitive::SpriteIndex(n as usize))
        .collect::<Vec<SpriteRenderPrimitive>>();

    let sprite_index_sampler = {
        Sampler {
            input: vec![0., 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4],
            function: InterpolationFunction::Step,
            output: sprite_indicies,
        }
    };

    let sprite_sheet_sampler = Sampler {
        input: vec![0., 2.3],
        function: InterpolationFunction::Step,
        output: vec![SpriteRenderPrimitive::SpriteSheet(sprite_sheet_id)],
    };

    let loader = world.write_resource::<Loader>();
    let sampler_animation_handle =
        loader.load_from_data(sprite_index_sampler, (), &world.read_resource());
    let sprite_sheet_sampler_animation_handle =
        loader.load_from_data(sprite_sheet_sampler, (), &world.read_resource());

    let animation = Animation {
        nodes: vec![
            (
                0,
                SpriteRenderChannel::SpriteSheet,
                sprite_sheet_sampler_animation_handle,
            ),
            (
                0,
                SpriteRenderChannel::SpriteIndex,
                sampler_animation_handle,
            ),
        ],
    };
    loader.load_from_data(animation, (), &world.read_resource())
}

pub fn brown_bat(world: &mut World, sprite_sheet_id: u64) -> Handle<Animation<SpriteRender>> {
    let sprite_indicies = (6..11)
        .into_iter()
        .map(|n| SpriteRenderPrimitive::SpriteIndex(n))
        .collect::<Vec<SpriteRenderPrimitive>>();

    let sprite_index_sampler = {
        Sampler {
            input: vec![0., 0.1, 0.2, 0.3, 0.4],
            function: InterpolationFunction::Step,
            output: sprite_indicies,
        }
    };

    let sprite_sheet_sampler = Sampler {
        input: vec![0.],
        function: InterpolationFunction::Step,
        output: vec![SpriteRenderPrimitive::SpriteSheet(sprite_sheet_id)],
    };

    let loader = world.write_resource::<Loader>();
    let sampler_animation_handle =
        loader.load_from_data(sprite_index_sampler, (), &world.read_resource());
    let sprite_sheet_sampler_animation_handle =
        loader.load_from_data(sprite_sheet_sampler, (), &world.read_resource());

    let animation = Animation {
        nodes: vec![
            (
                0,
                SpriteRenderChannel::SpriteSheet,
                sprite_sheet_sampler_animation_handle,
            ),
            (
                0,
                SpriteRenderChannel::SpriteIndex,
                sampler_animation_handle,
            ),
        ],
    };
    loader.load_from_data(animation, (), &world.read_resource())
}
