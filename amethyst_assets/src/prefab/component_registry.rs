use amethyst_core::ecs::{
    storage::{Archetype, Component, ComponentTypeId, ComponentWriter, Components},
    world::EntityHasher,
    Entity, Resources,
};
use legion_prefab::ComponentRegistration;
use prefab_format::ComponentTypeUuid;
use std::collections::HashMap;

use legion_prefab::{CopyCloneImpl, SpawnCloneImpl, SpawnCloneImplHandlerSet, SpawnInto};
use std::ops::Range;

use fnv::{FnvBuildHasher, FnvHashMap};

pub struct ComponentRegistryBuilder {
    components: FnvHashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: FnvHashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistryBuilder {
    pub fn new() -> Self {
        ComponentRegistryBuilder {
            components: Default::default(),
            components_by_uuid: Default::default(),
            spawn_handler_set: SpawnCloneImplHandlerSet::new(),
        }
    }

    pub fn auto_register_components(mut self) -> Self {
        let comp_registrations = legion_prefab::iter_component_registrations();

        for registration in comp_registrations {
            self = self.register_component(registration);
        }

        self
    }

    pub fn register_component(mut self, registration: &ComponentRegistration) -> Self {
        self.components
            .insert(registration.component_type_id(), registration.clone());
        self.components_by_uuid
            .insert(*registration.uuid(), registration.clone());
        self
    }

    pub fn add_spawn_mapping_into<FromT: Component + Clone + Into<IntoT>, IntoT: Component>(
        mut self,
    ) -> Self {
        self.spawn_handler_set.add_mapping_into::<FromT, IntoT>();
        self
    }

    pub fn add_spawn_mapping<FromT: Component + Clone + SpawnInto<IntoT>, IntoT: Component>(
        mut self,
    ) -> Self {
        self.spawn_handler_set.add_mapping::<FromT, IntoT>();
        self
    }

    pub fn add_spawn_mapping_closure<FromT, IntoT, F>(&mut self, clone_fn: F)
    where
        FromT: Component,
        IntoT: Component,
        F: Fn(
                &Resources,                             // resources
                Range<usize>,                           // src_entity_range
                &Archetype,                             // src_arch
                &Components,                            // src_components
                &mut ComponentWriter<IntoT>,            // dst
                fn(&mut ComponentWriter<IntoT>, IntoT), // push_fn
            ) + Send
            + Sync
            + 'static,
    {
        self.spawn_handler_set
            .add_mapping_closure::<FromT, _, _>(clone_fn);
    }

    pub fn build(self) -> ComponentRegistry {
        ComponentRegistry {
            components: self.components,
            components_by_uuid: self.components_by_uuid,
            spawn_handler_set: self.spawn_handler_set,
        }
    }
}

pub struct ComponentRegistry {
    components: FnvHashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: FnvHashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistry {
    pub fn components(&self) -> &FnvHashMap<ComponentTypeId, ComponentRegistration> {
        &self.components
    }

    pub fn components_by_uuid(&self) -> &FnvHashMap<ComponentTypeUuid, ComponentRegistration> {
        &self.components_by_uuid
    }

    pub fn copy_clone_impl(&self) -> CopyCloneImpl<FnvBuildHasher> {
        CopyCloneImpl::new(&self.components)
    }

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
