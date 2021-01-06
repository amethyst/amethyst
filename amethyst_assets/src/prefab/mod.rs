mod importers;
pub use importers::PrefabImporter;

mod assets;
pub use assets::{Prefab, RawPrefab};

mod system;
pub use system::{prefab_spawning_tick, RawPrefabMapping, RootPrefabs};

mod component_registry;
pub use component_registry::{ComponentRegistry, ComponentRegistryBuilder};
pub use legion_prefab::{register_component_type, ComponentRegistration};

mod processor;

// register core components
register_component_type!(amethyst_core::transform::Transform);
register_component_type!(amethyst_core::transform::TransformValues);
