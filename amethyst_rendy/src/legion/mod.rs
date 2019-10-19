use crate::{
    camera::{ActiveCamera, LegionActiveCamera},
    types::Backend,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{self as specs, SystemData},
    legion::{dispatcher::DispatcherBuilder, sync::SyncDirection, LegionState, LegionSyncBuilder},
    shrev::EventChannel,
    SystemBundle,
};

use amethyst_error::Error;
use amethyst_window::Event;
use derivative::Derivative;
use rendy::factory::Factory;
use std::marker::PhantomData;

pub mod bundle;
pub mod pass;
pub mod plugins;
pub mod sprite_visibility;
pub mod submodules;
pub mod system;
pub mod visibility;

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Syncer<B: Backend>(PhantomData<B>);
impl<B: Backend> LegionSyncBuilder for Syncer<B> {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        world: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    ) {
        crate::system::SetupData::setup(specs_world);

        world.add_component_sync_with(
            |direction,
             bimap,
             specs: Option<&mut ActiveCamera>,
             legion: Option<&mut LegionActiveCamera>| (None, None),
        );

        world.add_component_sync::<crate::SpriteRender>();
        world.add_component_sync::<crate::visibility::BoundingSphere>();
        world.add_component_sync::<crate::Camera>();
        world.add_component_sync::<crate::Transparent>();
        world.add_component_sync::<crate::resources::Tint>();
        world.add_component_sync::<crate::light::Light>(); // TODO: This causes chunk index out of bounds, why?
        world.add_component_sync::<crate::debug_drawing::DebugLinesComponent>();
        world.add_component_sync::<crate::skinning::JointTransforms>();
        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();
        world.add_component_sync::<crate::visibility::BoundingSphere>();

        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();

        world.add_resource_sync::<AssetStorage<crate::mtl::Material>>();
        world.add_resource_sync::<AssetStorage<crate::Mesh>>();
        world.add_resource_sync::<AssetStorage<crate::Texture>>();
        world.add_resource_sync::<AssetStorage<crate::sprite::SpriteSheet>>();

        world.add_resource_sync::<amethyst_assets::HotReloadStrategy>();
        world.add_resource_sync::<rendy::command::QueueId>();

        world.add_resource_sync::<crate::visibility::Visibility>();
        world.add_resource_sync::<crate::MaterialDefaults>();

        world.add_resource_sync::<amethyst_assets::Loader>();

        world.add_resource_sync::<Factory<B>>();

        world.add_resource_sync::<crate::debug_drawing::DebugLines>();

        // From window, but we sync here cuz lazy
        world.add_resource_sync::<amethyst_window::ScreenDimensions>();
        world.add_resource_sync::<amethyst_window::Window>();
        world.add_resource_sync::<EventChannel<Event>>();
    }
}
