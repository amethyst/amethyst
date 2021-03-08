use std::{collections::HashMap, ops::Range};

use amethyst_core::ecs::{
    storage::{Archetype, Component, ComponentTypeId, ComponentWriter, Components},
    world::EntityHasher,
    Entity, Resources,
};
use fnv::{FnvBuildHasher, FnvHashMap};
use legion_prefab::{
    ComponentRegistration, CopyCloneImpl, SpawnCloneImpl, SpawnCloneImplHandlerSet, SpawnInto,
};
use prefab_format::ComponentTypeUuid;

/// registers prefab components that can be created by prefab
/// use register_component_type! macro to add new components
#[derive(Default)]
pub struct ComponentRegistryBuilder {
    components: FnvHashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: FnvHashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistryBuilder {
    /// registers components that can be spawned, called by LoaderBundle
    pub fn auto_register_components(mut self) -> Self {
        let comp_registrations = legion_prefab::iter_component_registrations();

        for registration in comp_registrations {
            self = self.register_component(registration);
        }

        self
    }

    /// registers a single component to this registry
    pub fn register_component(mut self, registration: &ComponentRegistration) -> Self {
        self.components
            .insert(registration.component_type_id(), registration.clone());
        self.components_by_uuid
            .insert(*registration.uuid(), registration.clone());
        self
    }

    /// adds a mapping between prefab component and spawned component
    pub fn add_spawn_mapping_into<FromT: Component + Clone + Into<IntoT>, IntoT: Component>(
        mut self,
    ) -> Self {
        self.spawn_handler_set.add_mapping_into::<FromT, IntoT>();
        self
    }

    /// adds a mapping between prefab component and spawned for already initialized component memory
    pub fn add_spawn_mapping<FromT: Component + Clone + SpawnInto<IntoT>, IntoT: Component>(
        mut self,
    ) -> Self {
        self.spawn_handler_set.add_mapping::<FromT, IntoT>();
        self
    }

    /// adds a mapping between prefab and spawned component that can be modified with a closure
    pub fn add_spawn_mapping_closure<FromT, IntoT, F>(&mut self, clone_fn: F)
    where
        FromT: Component,
        IntoT: Component,
        F: Fn(
                &Resources,                                 // resources
                Range<usize>,                               // src_entity_range
                &Archetype,                                 // src_arch
                &Components,                                // src_components
                &mut ComponentWriter<'_, IntoT>,            // dst
                fn(&mut ComponentWriter<'_, IntoT>, IntoT), // push_fn
            ) + Send
            + Sync
            + 'static,
    {
        self.spawn_handler_set
            .add_mapping_closure::<FromT, _, _>(clone_fn);
    }

    /// builds the component registry with spawn mappings
    pub fn build(self) -> ComponentRegistry {
        ComponentRegistry {
            components: self.components,
            components_by_uuid: self.components_by_uuid,
            spawn_handler_set: self.spawn_handler_set,
        }
    }
}

/// stores information about how to construct components from prefabs
pub struct ComponentRegistry {
    components: FnvHashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: FnvHashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistry {
    /// returns reference to the components map
    pub fn components(&self) -> &FnvHashMap<ComponentTypeId, ComponentRegistration> {
        &self.components
    }

    /// returns reference to the components mapped by their UUID
    pub fn components_by_uuid(&self) -> &FnvHashMap<ComponentTypeUuid, ComponentRegistration> {
        &self.components_by_uuid
    }

    /// allows to trivially copy components in to a world
    pub fn copy_clone_impl(&self) -> CopyCloneImpl<'_, FnvBuildHasher> {
        CopyCloneImpl::new(&self.components)
    }

    /// the full spawn clone which will map components by their spawn handlers
    pub fn spawn_clone_impl<'a, 'b, 'c>(
        &'a self,
        resources: &'b Resources,
        entity_map: &'c HashMap<Entity, Entity, EntityHasher>,
    ) -> SpawnCloneImpl<'a, 'a, 'b, 'c, fnv::FnvBuildHasher> {
        SpawnCloneImpl::new(
            &self.spawn_handler_set,
            &self.components,
            resources,
            entity_map,
        )
    }
}
