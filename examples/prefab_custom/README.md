## Prefab Custom

Create a `PrefabData` and instantiates multiple entities with different components using the `Prefab` system.

No window is created for this demo, instead you should see debugging information printed to the console similar to:

```log
[INFO][amethyst::app] Initializing Amethyst...
[INFO][amethyst::app] Version: 0.15.0
[INFO][amethyst::app] Platform: x86_64-apple-darwin
[INFO][amethyst::app] Amethyst git commit: 2aafa4ff85ed4d5636b34cea049c4f3804322860
[INFO][amethyst::app] Rustc version: 1.44.0 Stable
[INFO][amethyst::app] Rustc git commit: 49cae55760da0a43428eba73abcb659bb70cf2e4
Prefab
======
PrefabEntity { parent: None, data: Some(Player { name: Named { name: "Zero" }, position: Some(Position(1.0, 2.0, 3.0)) }) }
PrefabEntity { parent: Some(0), data: Some(Weapon { weapon_type: Sword, position: Some(Position(4.0, 5.0, 6.0)) }) }

Entities
========

| Entity                   | Handle<Prefab<CustomPrefabData>>> | Parent                   | Position                | Player                 | Weapon |
| ------------------------ | --------------------------------- | ------------------------ | ----------------------- | ---------------------- | ------ |
| Entity(0, Generation(1)) | Handle { id: 0 }                  | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(1, Generation(1)) | None                              | Entity(0, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |
[INFO][amethyst::app] Engine is shutting down
```
