mod importers;
use amethyst_core::transform::Parent;
pub use importers::PrefabImporter;

mod assets;
pub use assets::Prefab;

pub(crate) mod system;

mod component_registry;
pub use component_registry::{ComponentRegistry, ComponentRegistryBuilder};
pub use legion_prefab::{self, register_component_type, ComponentRegistration};
pub use serde_diff::{self, SerdeDiff};

mod processor;

// register core components
register_component_type!(amethyst_core::transform::Transform);
register_component_type!(amethyst_core::transform::TransformValues);
register_component_type!(Parent);
