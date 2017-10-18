//! Scene graph system and types

use cgmath::Matrix4;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use specs::{Entities, Entity, Join, System, WriteStorage};
use hibitset::{BitSet, BitSetLike};
use transform::{LocalTransform, Transform, Parent};

/// Handles updating `Transform` components based on the `LocalTransform`
/// component and parents.
#[derive(Default)]
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<Entity>,

    init: BitSet,
    frame_init: BitSet,

    dead: HashSet<Entity>,
    remove_parent: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            indices: HashMap::default(),
            sorted: Vec::new(),
            init: BitSet::new(),
            frame_init: BitSet::new(),
            dead: HashSet::default(),
            remove_parent: Vec::new(),
        }
    }

    fn remove(&mut self, index: usize) {
        let entity = self.sorted[index];
        self.sorted.swap_remove(index);
        if let Some(swapped) = self.sorted.get(index) {
            self.indices.insert(*swapped, index);
        }
        self.indices.remove(&entity);
        self.init.remove(index as u32);
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, LocalTransform>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Transform>,
    );
    fn run(&mut self, (entities, mut locals, mut parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        let mut parented = Vec::new();
        let mut noparents = Vec::new();
        let mut initted = Vec::new();

        // Clear dirty flags on `Transform` storage, before updates go in
        (&mut globals).open().1.clear_flags();

        {
            for (entity, parent) in (&*entities, parents.open().1).join() {
                if parent.entity == entity {
                    self.remove_parent.push(entity);   
                }
            }

            for entity in self.remove_parent.iter() {
                eprintln!("Entity was its own parent: {:?}", entity);
                parents.remove(*entity);
            }

            self.remove_parent.clear();
        }

        {
            // Checks for entities with a modified local transform or a modified parent, but isn't initialized yet.
            let filter = locals.open().0 & parents.open().0 & !&self.init; // has a local, parent, and isn't initialized.
            for (entity, _) in (&*entities, &filter).join() {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push(entity);
                self.frame_init.add(entity.id());

                initted.push(entity.id());
            }

            println!("Initialized: {:?}", initted);
        }

        {
            let locals_flagged = locals.open().1;

            // Compute transforms without parents.
            for (_entity, local, global, _) in (&*entities, locals_flagged, &mut globals, !&parents).join() {
                global.0 = local.matrix();
                noparents.push(_entity.id());
                #[cfg(debug_assertions)]
                {
                    if global.0 != global.0 {
                        panic!("Entity {:?} has an invalid transform (NaN data)", _entity);
                    }
                }
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let entity = self.sorted[index];
            let local_dirty = locals.open().1.flagged(entity);
            let parent_dirty = parents.open().1.flagged(entity);

            match (
                parents.get(entity),
                locals.get(entity),
                self.dead.contains(&entity),
            ) {
                (Some(parent), Some(local), false) => {
                    // Make sure this iteration isn't a child before the parent.
                    if parent_dirty {
                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&parent.entity) {
                            if parent_index > &index {
                                swap = Some(*parent_index);
                            }
                        }

                        if let Some(p) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, index);
                            self.indices.insert(parent.entity, index);
                            self.indices.insert(entity, p);

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    // Kill the entity is the parent is dead.
                    if self.dead.contains(&parent.entity) || !entities.is_alive(parent.entity) {
                        self.remove(index);
                        let _ = entities.delete(entity);
                        self.dead.insert(entity);

                        // Re-try index because swapped with last element.
                        continue;
                    }

                    if local_dirty || parent_dirty || globals.open().1.flagged(parent.entity) {
                        let combined_transform = if let Some(parent_global) =
                            globals.get(parent.entity)
                        {
                            (Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())).into()
                        } else {
                            local.matrix()
                        };
                        
                        parented.push(((local_dirty, parent_dirty, globals.open().1.flagged(parent.entity)), entity.id()));
                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform;
                        }
                    }
                }
                (_, _, dead @ _) => { // This entity should not be in the sorted list, so remove it.
                    self.remove(index);

                    if !dead && !entities.is_alive(entity) {
                        self.dead.insert(entity);
                    }

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            index += 1;
        }

        println!("with parent updated: {:?}", parented);
        println!("no parent updated: {:?}", noparents);

        (&mut locals).open().1.clear_flags();
        (&mut parents).open().1.clear_flags();

        for bit in (&self.frame_init).iter() {
            self.init.add(bit);
        }
        self.frame_init.clear();
        self.dead.clear();
    }
}

#[cfg(test)]
mod tests {
    use specs::World;
    use transform::{Parent, LocalTransform, Transform, TransformSystem};
    use cgmath::{Decomposed, Quaternion, Vector3, Matrix4};
    use shred::RunNow;
    //use quickcheck::{Arbitrary, Gen};

    // If this works, then all other tests should work.
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

    fn transform_world<'a, 'b>() -> (World, TransformSystem) {
        let mut world = World::new();
        world.register::<LocalTransform>();
        world.register::<Transform>();
        world.register::<Parent>();

        (world, TransformSystem::new())
    }

    fn together(transform: Transform, local: LocalTransform) -> [[f32; 4]; 4] {
        (Matrix4::from(transform.0) * Matrix4::from(local.matrix())).into()
    }

    // Basic default LocalTransform -> Transform (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut system) = transform_world();

        let mut transform = LocalTransform::default();
        transform.translation = [0.0, 0.0, 0.0];
        transform.rotation = [1.0, 0.0, 0.0, 0.0];

        let e1 = world.create_entity()
            .with(transform)
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = Transform::default().into();
        assert_eq!(a1, a2);
    }

    // Basic sanity check for LocalTransform -> Transform, no parent relationships
    //
    // Should just put the value of the LocalTransform matrix into the Transform component.
    #[test]
    fn basic() {
        let (mut world, mut system) = transform_world();

        let mut local = LocalTransform::default();
        local.translation = [5.0, 5.0, 5.0];
        local.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * LocalTransform -> Transform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut system) = transform_world();

        let mut local1 = LocalTransform::default();
        local1.translation = [5.0, 5.0, 5.0];
        local1.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = [5.0, 5.0, 5.0];
        local2.rotation = [1.0, 0.5, 0.5, 0.0];

        let e2 = world.create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = LocalTransform::default();
        local3.translation = [5.0, 5.0, 5.0];
        local3.rotation = [1.0, 0.5, 0.5, 0.0];

        let e3 = world.create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .with(Parent { entity: e2 })
            .build();

        system.run_now(&mut world.res);

        let transforms = world.read::<Transform>();

        let transform1 = {
            // First entity (top level parent)
            let transform1 = transforms.get(e1).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform1.into();
            let a2: [[f32; 4]; 4] = local1.matrix().into();
            assert_eq!(a1, a2);
            transform1
        };

        let transform2 = {
            let transform2 = transforms.get(e2).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform2.into();
            let a2: [[f32; 4]; 4] = together(transform1, local2);
            assert_eq!(a1, a2);
            transform2
        };

        {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
        };
    }

    // Test Parent * LocalTransform -> Transform (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut system) = transform_world();

        let mut local3 = LocalTransform::default();
        local3.translation = [5.0, 5.0, 5.0];
        local3.rotation = [1.0, 0.5, 0.5, 0.0];

        let e3 = world.create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = [5.0, 5.0, 5.0];
        local2.rotation = [1.0, 0.5, 0.5, 0.0];

        let e2 = world.create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .build();

        let mut local1 = LocalTransform::default();
        local1.translation = [5.0, 5.0, 5.0];
        local1.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        {
            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent { entity: e1 }); 
            parents.insert(e3, Parent { entity: e2 }); 
        }

        system.run_now(&mut world.res);

        let transforms = world.read::<Transform>();

        let transform1 = {
            // First entity (top level parent)
            let transform1 = transforms.get(e1).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform1.into();
            let a2: [[f32; 4]; 4] = local1.matrix().into();
            assert_eq!(a1, a2);
            transform1
        };

        let transform2 = {
            let transform2 = transforms.get(e2).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform2.into();
            let a2: [[f32; 4]; 4] = together(transform1, local2);
            assert_eq!(a1, a2);
            transform2
        };

        {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
        };
    }

    #[test]
    #[should_panic]
    fn nan_transform() {
        let (mut world, mut system) = transform_world();

        let mut local = LocalTransform::default();
        // Release the indeterminate forms!
        local.translation = [0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0];

        world.create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);
    }

    #[test]
    fn entity_is_parent() {
        let (mut world, mut system) = transform_world();

        let e3 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        world.write::<Parent>().insert(e3, Parent { entity: e3 });
        system.run_now(&mut world.res);

        let parents = world.read::<Parent>();
        assert_eq!(parents.get(e3), None)
    }

    #[test]
    fn parent_removed() {
        let (mut world, mut system) = transform_world();

        let e1 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        let e2 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e1 })
            .build();

        let e3 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        let e4 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e3 })
            .build();

        let e5 = world.create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e4 })
            .build();

        let _ = world.delete_entity(e1);
        system.run_now(&mut world.res);
        world.maintain();

        assert_eq!(world.is_alive(e1), false);
        assert_eq!(world.is_alive(e2), false);

        let _ = world.delete_entity(e3);
        system.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();

        assert_eq!(world.is_alive(e3), false);
        assert_eq!(world.is_alive(e4), false);
        assert_eq!(world.is_alive(e5), false);
    }

    /*
    struct LocalShrinker {
        transform: LocalTransform,
    }

    fn check_identity(transform: &LocalTransform) -> bool {
        LocalTransform::default() == *transform
    }

    fn pretty_matrix(matrix: [[f32; 4]; 4]) -> String {
        format!("[\n\t[{}, {}, {}, {}],\n\t[{}, {}, {}, {}],\n\t[{}, {}, {}, {}],\n\t[{}, {}, {}, {}],\n]",
                matrix[0][0], matrix[0][1], matrix[0][2], matrix[0][3],
                matrix[1][0], matrix[1][1], matrix[1][2], matrix[1][3],
                matrix[2][0], matrix[2][1], matrix[2][2], matrix[2][3],
                matrix[3][0], matrix[3][1], matrix[3][2], matrix[3][3],)
    }

    impl Iterator for LocalShrinker {
        type Item = LocalTransform;
        fn next(&mut self) -> Option<Self::Item> {
            let translationx = self.transform.translation[0].shrink().next().unwrap_or(0.0);
            let translationy = self.transform.translation[1].shrink().next().unwrap_or(0.0);    
            let translationz = self.transform.translation[2].shrink().next().unwrap_or(0.0);    

            let mut rotw = self.transform.rotation[0].shrink().next().unwrap_or(1.0);
            if rotw < 1.0 { rotw = 1.0; }

            let rotx = self.transform.rotation[1].shrink().next().unwrap_or(0.0);
            let roty = self.transform.rotation[2].shrink().next().unwrap_or(0.0);
            let rotz = self.transform.rotation[3].shrink().next().unwrap_or(0.0);

            let mut scalex = self.transform.scale[0].shrink().next().unwrap_or(1.0);
            if scalex < 1.0 { scalex = 1.0; }
            let mut scaley = self.transform.scale[1].shrink().next().unwrap_or(1.0);    
            if scaley < 1.0 { scaley = 1.0; }
            let mut scalez = self.transform.scale[2].shrink().next().unwrap_or(1.0);
            if scalez < 1.0 { scalez = 1.0; }

            let transform = LocalTransform {
                translation: [translationx, translationy, translationz],
                rotation: [rotw, rotx, roty, rotz],
                scale: [scalex, scaley, scalez],
            };
            
            self.transform = transform;

            if self.transform != self.transform || check_identity(&self.transform) {
                None
            }
            else {
                Some(self.transform.clone())
            }
        }
    }

    impl Arbitrary for LocalTransform {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            LocalTransform {
                translation: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)],
                rotation: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)],
                scale: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)],
            }
        }

        fn shrink(&self) -> Box<Iterator<Item=Self>> {
            if self != self || check_identity(self) {
                Box::new(::quickcheck::empty_shrinker())
            }
            else {
                Box::new(LocalShrinker { transform: self.clone() })
            }
        }
    }

    type Phase = (Vec<u8>, Vec<(Option<u32>, LocalTransform)>);

    fn insert_into((delete, list): &Phase, LocalTransform)>, world: &mut World) -> Vec<(Entity, Option<Entity>)> {
        let entities = world.create_iter().take(list.len()).collect::<Vec<Entity>>();
        let mut locals = world.write::<LocalTransform>();
        let mut transforms = world.write::<Transform>();
        let mut parents = world.write::<Parent>();

        let mut result = Vec::new();

        println!("valid relations: {:?}", list.iter().enumerate().filter_map(|(index, &(relation, _))| {
            match relation {
                Some(relation) if entities.len() <= relation as usize => Some((index, relation)),
                _ => None,
            }
        }).collect::<Vec<(usize, u32)>>());

        let mut inserted = Vec::new();

        for (index, &(ref relation, ref transform)) in (&list).iter().enumerate() {
            inserted.push(index);
            if entities.len() <= index as usize {
                continue;
            }

            locals.insert(entities[index], transform.clone()); 
            transforms.insert(entities[index], Transform::default());

            let mut parent = None;
            if let Some(relation) = *relation {
                if entities.len() <= relation as usize {
                    continue;
                }

                parents.insert(entities[index], Parent { entity: entities[relation as usize] });
                parent = Some(entities[relation as usize]);
            }

            result.push((entities[index], parent));
        }

        for (index, num) in delete.iter().enumerate() {
            if num > 200 {
                world.delete_entity(entities[index]);
            }
        }
        
        println!("inserting for indices {:?}", inserted);

        result
    }


    quickcheck! {
        fn fuzzy_test(list: Vec<Phase>) -> bool {
            let result = ::std::panic::catch_unwind(|| {
                println!();
                println!("Input: {:#?}", list);
                let (mut world, mut system) = transform_world();
                let entities = insert_into(&list, &mut world);
                println!("Entities: {:?}", entities);

                system.run_now(&mut world.res);

                let locals = world.read::<LocalTransform>();
                let transforms = world.read::<Transform>();
                let parents = world.read::<Parent>();

                for &(entity, parent) in entities.iter() {
                    let local_entity = match locals.get(entity) {
                        Some(transform) => transform,
                        None => {
                            println!("No local transform for {:?}", entity); 
                            println!("Failed");
                            return false;
                        },
                    };

                    let world_entity = match transforms.get(entity) {
                        Some(transform) => transform,
                        None => {
                            println!("No world transform for {:?}", entity); 
                            println!("Failed");
                            return false;
                        },
                    };

                    match parent {
                        Some(parent_entity) if parents.get(entity).is_some() => {
                            println!("Has parent");
                            let world_parent = match transforms.get(parent_entity) {
                                Some(transform) => transform,
                                None => {
                                    println!("No world transform for parent {:?}", parent); 
                                    println!("Failed");
                                    return false;
                                },
                            };

                            let combined = together(world_parent.clone(), local_entity.clone());

                            let result = combined == world_entity.0;
                            if !result {
                                println!("Incorrect transform for {:?} -> {:?}: correct: {}, actual: {}",
                                         parent,
                                         entity, 
                                         pretty_matrix(combined),
                                         pretty_matrix(world_entity.0)
                                        ); 
                                println!("Failed");
                                return false;
                            }
                        }
                        _ => {
                            println!("Has no parent");
                            let result = local_entity.matrix() == world_entity.0;
                            if !result {
                                println!("Incorrect transform for {:?}:\nlocal:{:?}\ncorrect:\n{}\nactual:\n{}",
                                         entity,
                                         local_entity,
                                         pretty_matrix(local_entity.matrix()),
                                         pretty_matrix(world_entity.0)
                                        ); 
                                println!("Failed");
                                return false;
                            }
                        }
                    }
                }

                println!("Succeeded");
                true
            });

            match result {
                Ok(b) => {
                    if b { println!("Succeeded"); } else { println!("Failed"); }
                    b
                }
                Err(_) => {
                    println!("Thread panicked on NaN data.");
                    println!("Succeeded");
                    true
                }
            }
        }
    }
    */
}
