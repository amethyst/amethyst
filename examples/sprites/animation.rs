use amethyst::assets::{Handle, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{Material, SpriteSheet};
use amethyst_animation::{
    Animation, InterpolationFunction, MaterialChannel, MaterialPrimitive, Sampler,
};

pub fn grey_bat(sprite_sheet: &SpriteSheet, world: &mut World) -> Handle<Animation<Material>> {
    let sprite_offsets = [5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 4]
        .into_iter()
        .map(|n| (&sprite_sheet.sprites[*n]).into())
        .collect::<Vec<MaterialPrimitive>>();

    let sprite_offset_sampler = {
        Sampler {
            input: vec![0., 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4],
            function: InterpolationFunction::Step,
            output: sprite_offsets,
        }
    };

    let texture_sampler = Sampler {
        input: vec![0., 2.3],
        function: InterpolationFunction::Step,
        output: vec![
            MaterialPrimitive::Texture(sprite_sheet.texture_id),
            MaterialPrimitive::Texture(sprite_sheet.texture_id),
        ],
    };

    let loader = world.write_resource::<Loader>();
    let sampler_animation_handle =
        loader.load_from_data(sprite_offset_sampler, (), &world.read_resource());
    let texture_animation_handle =
        loader.load_from_data(texture_sampler, (), &world.read_resource());

    let animation = Animation {
        nodes: vec![
            (0, MaterialChannel::AlbedoTexture, texture_animation_handle),
            (0, MaterialChannel::AlbedoOffset, sampler_animation_handle),
        ],
    };
    loader.load_from_data(animation, (), &world.read_resource())
}

pub fn brown_bat(sprite_sheet: &SpriteSheet, world: &mut World) -> Handle<Animation<Material>> {
    let sprite_offsets = sprite_sheet.sprites[6..11]
        .iter()
        .map(|sprite| sprite.into())
        .collect::<Vec<MaterialPrimitive>>();

    let sprite_offset_sampler = {
        Sampler {
            input: vec![0., 0.1, 0.2, 0.3, 0.4],
            function: InterpolationFunction::Step,
            output: sprite_offsets,
        }
    };

    let texture_sampler = Sampler {
        input: vec![0.],
        function: InterpolationFunction::Step,
        output: vec![MaterialPrimitive::Texture(sprite_sheet.texture_id)],
    };

    let loader = world.write_resource::<Loader>();
    let sampler_animation_handle =
        loader.load_from_data(sprite_offset_sampler, (), &world.read_resource());
    let texture_animation_handle =
        loader.load_from_data(texture_sampler, (), &world.read_resource());

    let animation = Animation {
        nodes: vec![
            (0, MaterialChannel::AlbedoTexture, texture_animation_handle),
            (0, MaterialChannel::AlbedoOffset, sampler_animation_handle),
        ],
    };
    loader.load_from_data(animation, (), &world.read_resource())
}
