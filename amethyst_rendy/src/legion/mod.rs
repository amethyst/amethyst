use crate::types::Backend;
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{ecs as specs, legion::sync, shred::DispatcherBuilder, SystemBundle};
use amethyst_error::Error;
use derivative::Derivative;
use rendy::factory::Factory;
use std::marker::PhantomData;
pub mod bundle;
pub mod system;

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct RenderLegionBundle<B: Backend>(PhantomData<B>);

impl<'a, 'b, B: Backend> SystemBundle<'a, 'b> for RenderLegionBundle<B> {
    fn build(
        self,
        world: &mut specs::World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Create the legion world

        let mut world = world.fetch_mut::<sync::LegionWorld>();
        world.add_component_sync::<crate::SpriteRender>();
        world.add_component_sync::<crate::visibility::BoundingSphere>();
        world.add_component_sync::<crate::Camera>();
        world.add_component_sync::<crate::Transparent>();
        world.add_component_sync::<crate::resources::Tint>();
        //world.add_component_sync::<crate::light::Light>(); // TODO: This causes chunk index out of bounds, why?
        world.add_component_sync::<crate::debug_drawing::DebugLinesComponent>();
        world.add_component_sync::<crate::skinning::JointTransforms>();
        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();

        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();

        world.add_resource_sync::<AssetStorage<crate::mtl::Material>>();
        world.add_resource_sync::<AssetStorage<crate::Mesh>>();
        world.add_resource_sync::<AssetStorage<crate::Texture>>();

        world.add_resource_sync::<amethyst_assets::HotReloadStrategy>();
        world.add_resource_sync::<rendy::command::QueueId>();

        world.add_resource_sync::<crate::visibility::Visibility>();
        world.add_resource_sync::<crate::MaterialDefaults>();

        world.add_resource_sync::<Factory<B>>();

        Ok(())
    }
}
