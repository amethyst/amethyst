//! Components for the transform processor.

pub use self::local_transform::Transform;
pub use self::parent::{HierarchyEvent, Parent, ParentHierarchy};
pub use self::transform::GlobalTransform;

mod local_transform;
mod parent;
mod transform;

/// Prefab component data for Transform
#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct TransformPrefabData {
    pub transform: Transform,
}

impl From<Transform> for TransformPrefabData {
    fn from(transform: Transform) -> Self {
        TransformPrefabData { transform }
    }
}
