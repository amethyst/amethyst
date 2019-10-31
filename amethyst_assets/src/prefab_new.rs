
struct BakedEntityData {
    data: Vec<()>
}

struct EntityData {
    id: Entity,
    components: Vec<Component>,
}
struct Prefab {
    root: Option<Entity>,
    objects: Vec<EntityData>,
}
struct PrefabObjectReference {
    prefab: Handle<Prefab>,
    object: Entity,
}

struct PrefabVariant {
    prefab: Handle<Prefab>,
    changes: (),
}

enum PrefabReference {
    PrefabObject(PrefabObjectReference),
    Prefab(Handle<Prefab>),
    PrefabVariant(PrefabVariant),
}