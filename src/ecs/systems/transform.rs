//! Scene graph system and types

use cgmath::Matrix4;
use config::Config;
use ecs::{Entities, Entity, Join, ReadStorage, System, WriteStorage};
use ecs::components::{LocalTransform, Transform, Child, Init};
use ecs::systems::SystemExt;
use error::Result;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};

/// Handles updating `Transform` components based on the `LocalTransform`
/// component and parents.
#[derive(Clone, Debug, Default)]
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<(Entity, Entity)>,
    /// New entities in the current update.
    new: Vec<Entity>,
    /// Entities that have been removed in current frame.
    dead: HashSet<Entity>,
    /// Child entities that were dirty.
    dirty: HashSet<Entity>,
    /// Prevent circular infinite loops with parents.
    swapped: HashSet<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem::default()
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (Entities<'a>,
     ReadStorage<'a, LocalTransform>,
     ReadStorage<'a, Child>,
     WriteStorage<'a, Init>,
     WriteStorage<'a, Transform>);

    fn run(&mut self, (entities, locals, children, mut init, mut globals): Self::SystemData) {

        // Checks for entities with a local transform and parent, but no
        // `Init` component.
        for (entity, _, child, _) in (&*entities, &locals, &children, !&init).join() {
            self.indices.insert(entity, self.sorted.len());
            self.sorted.push((entity, child.parent()));
            self.new.push(entity);
        }

        // Deletes entities whose parents aren't alive.
        for &(entity, _) in &self.sorted {
            if let Some(child) = children.get(entity) {
                if !entities.is_alive(child.parent()) || self.dead.contains(&child.parent()) {
                    entities.delete(entity);
                    self.dead.insert(entity);
                }
            }
        }

        // Adds an `Init` component to the entity.
        for entity in self.new.drain(..) {
            init.insert(entity, Init);
        }

        {
            // Compute transforms without parents.
            for (ent, local, global, _) in (&*entities, &locals, &mut globals, !&children).join() {
                if local.is_dirty() {
                    self.dirty.insert(ent);
                    global.0 = local.matrix();
                    local.flag(false);
                }
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];

            match (children.get(entity), locals.get(entity), self.dead.contains(&entity)) {
                (Some(child), Some(local), false) => {
                    // Make sure the transform is also dirty if the parent has changed.
                    if child.is_dirty() && !self.swapped.contains(&entity) {
                        if child.parent() != parent_entity {
                            self.sorted[index] = (entity, child.parent());
                        }

                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&child.parent()) {
                            if parent_index > &index {
                                swap = Some(*parent_index);
                            }
                        }

                        if let Some(p) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, index);
                            self.indices.insert(child.parent(), index);
                            self.indices.insert(entity, p);
                            self.swapped.insert(entity);

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    if local.is_dirty() || child.is_dirty() ||
                       self.dirty.contains(&child.parent()) {
                        let combined_transform = if let Some(parent_global) =
                            globals.get(child.parent()) {
                            (Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())).into()
                        } else {
                            local.matrix()
                        };

                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform;
                        }

                        local.flag(false);
                        child.flag(false);
                        self.dirty.insert(entity);
                    }
                }
                _ => {
                    self.sorted.swap_remove(index); // swap with last to prevent shift
                    if let Some(swapped) = self.sorted.get(index) {
                        self.indices.insert(swapped.0, index);

                        // Make sure to check for parent swap next iteration
                        if let Some(parent) = children.get(swapped.0) {
                            parent.flag(true);
                        }
                    }
                    self.indices.remove(&entity);
                    init.remove(entity);

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            index += 1;
        }

        self.dirty.clear();
        self.dead.clear();
        self.swapped.clear();
    }
}

impl SystemExt for TransformSystem {
    fn build(_: &Config) -> Result<TransformSystem> {
        Ok(TransformSystem::default())
    }

    fn register(world: &mut World) {
        world.register::<Child>();
        world.register::<Init>();
        world.register::<LocalTransform>();
        world.register::<Transform>();
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Decomposed, Quaternion, Vector3, Matrix4};
    use ecs::components::{LocalTransform, Transform};

    #[test]
    fn transform_matrix() {
        let mut transform = LocalTransform::default();
        transform.translation = [5.0, 2.0, -0.5];
        transform.rotation = [1.0, 0.0, 0.0, 0.0];
        transform.scale = [2.0, 2.0, 2.0];

        let decomposed = Decomposed {
            rot: Quaternion::from(transform.rotation),
            disp: Vector3::from(transform.translation),
            scale: 2.0,
        };

        let matrix = transform.matrix();
        let cg_matrix: Matrix4<f32> = decomposed.into();
        let cg_matrix: [[f32; 4]; 4] = cg_matrix.into();

        assert_eq!(matrix, cg_matrix);
    }

    #[test]
    fn into_from() {
        let transform = Transform::default();
        let primitive: [[f32; 4]; 4] = transform.into();
        assert_eq!(primitive, transform.0);

        let transform: Transform = primitive.into();
        assert_eq!(primitive, transform.0);
    }
}
