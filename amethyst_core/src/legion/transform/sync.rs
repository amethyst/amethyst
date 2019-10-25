use crate::{
    ecs as specs,
    legion::{
        sync::{EntitiesBimapRef, SyncDirection, SyncerTrait},
        LegionState,
    },
    math::{self as na, Vector3},
    transform::Transform,
};
use legion::prelude as l;
use legion_transform::components as ltc;
use std::marker::PhantomData;

#[derive(Default)]
pub struct TransformSyncer;
impl SyncerTrait for TransformSyncer {
    fn setup(&self, world: &mut specs::World) {}

    fn sync(
        &self,
        specs_world: &mut specs::World,
        legion_state: &mut LegionState,
        direction: SyncDirection,
    ) {
        use specs::{Join, SystemData};

        let (bimap, entities, mut transforms) = <(
            specs::Read<'_, EntitiesBimapRef>,
            specs::Entities<'_>,
            specs::WriteStorage<'_, Transform>,
        )>::fetch(specs_world);

        let bimap = bimap.read().unwrap();

        match direction {
            SyncDirection::SpecsToLegion => {
                for (specs_entity, transform) in (&entities, &transforms).join() {
                    let legion_entity = bimap.get_by_right(&specs_entity).unwrap();
                    specs_to_legion(transform, *legion_entity, &mut legion_state.world).unwrap();
                }
            }
            SyncDirection::LegionToSpecs => {
                for (legion_entity, specs_entity) in bimap.iter() {
                    let transform = {
                        if let Some(transform) = transforms.get_mut(*specs_entity) {
                            transform
                        } else {
                            transforms.insert(*specs_entity, Transform::default());
                            transforms.get_mut(*specs_entity).unwrap()
                        }
                    };
                    legion_to_specs(*legion_entity, &mut legion_state.world, transform);
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum LegionTransformSetScale {
    NonUniform(ltc::NonUniformScale),
    Uniform(ltc::Scale),
}

#[derive(Default, Clone)]
pub struct LegionTransformSet {
    translation: Option<ltc::Translation>,
    rotation: Option<ltc::Rotation>,
    scale: Option<LegionTransformSetScale>,
}

pub fn legion_to_specs(
    legion_entity: l::Entity,
    world: &mut l::World,
    transform: &mut Transform,
) -> Result<(), ()> {
    let set = LegionTransformSet {
        translation: world
            .get_component::<ltc::Translation>(legion_entity)
            .map(|v| *v),
        rotation: world
            .get_component::<ltc::Rotation>(legion_entity)
            .map(|v| *v),
        scale: {
            let scale = world
                .get_component::<ltc::Scale>(legion_entity)
                .map(|v| LegionTransformSetScale::Uniform(*v));
            if scale.is_none() {
                world
                    .get_component::<ltc::NonUniformScale>(legion_entity)
                    .map(|v| LegionTransformSetScale::NonUniform(*v))
            } else {
                scale
            }
        },
    };

    *transform = convert_legion_to_specs(set);

    Ok(())
}

pub fn specs_to_legion(
    transform: &Transform,
    legion_entity: l::Entity,
    world: &mut l::World,
) -> Result<(), ()> {
    let set = convert_specs_to_legion(transform)?;

    if let Some(translation) = set.translation {
        world.add_component(legion_entity, translation);
    } else {
        world.remove_component::<ltc::Translation>(legion_entity);
    }

    if let Some(rotation) = set.rotation {
        world.add_component(legion_entity, rotation);
    } else {
        world.remove_component::<ltc::Rotation>(legion_entity);
    }

    if let Some(scale) = set.scale {
        match scale {
            LegionTransformSetScale::Uniform(value) => {
                world.add_component(legion_entity, value);
                world.remove_component::<ltc::NonUniformScale>(legion_entity);
            }
            LegionTransformSetScale::NonUniform(value) => {
                world.add_component(legion_entity, value);
                world.remove_component::<ltc::Scale>(legion_entity);
            }
        }
    }

    Ok(())
}

pub fn convert_legion_to_specs(set: LegionTransformSet) -> Transform {
    let mut transform = Transform::default();

    if let Some(translation) = set.translation {
        transform.set_translation(translation.vector.xyz());
    }

    if let Some(rotation) = set.rotation {
        transform.set_rotation(*rotation);
    }

    if let Some(scale) = set.scale {
        match scale {
            LegionTransformSetScale::Uniform(value) => {
                transform.set_scale(Vector3::new(*value, *value, *value))
            }
            LegionTransformSetScale::NonUniform(value) => transform.set_scale(*value),
        }
    }

    transform
}

pub fn convert_specs_to_legion(specs: &Transform) -> Result<LegionTransformSet, ()> {
    let scale = {
        let scale = specs.scale();
        if *scale != Vector3::new(1.0, 1.0, 1.0) {
            if scale.x == scale.y && scale.y == scale.z {
                Some(LegionTransformSetScale::Uniform(ltc::Scale::from(scale.x)))
            } else {
                Some(LegionTransformSetScale::NonUniform(
                    ltc::NonUniformScale::from(*scale),
                ))
            }
        } else {
            None
        }
    };

    let rotation = {
        let q = *specs.rotation();
        if q != *ltc::Rotation::identity() {
            Some(q.into())
        } else {
            None
        }
    };
    let translation = {
        if *specs.translation() != ltc::Translation::identity().vector.xyz() {
            Some(ltc::Translation::from(*specs.translation()))
        } else {
            None
        }
    };

    Ok(LegionTransformSet {
        translation,
        rotation,
        scale,
    })
}
