## Prefab Multi

Creates a `PrefabData` and instantiates an entity with multiple components using the `Prefab` system.

No window is created for this demo, instead you should see debugging information printed to the console similar to:

```log
[INFO][amethyst::app] Initializing Amethyst...
[INFO][amethyst::app] Version: 0.15.0
[INFO][amethyst::app] Platform: x86_64-apple-darwin
[INFO][amethyst::app] Amethyst git commit: 3de15588f1d4f7cd62d7253402749cfd26a42c0f
[INFO][amethyst::app] Rustc version: 1.44.0 Stable
[INFO][amethyst::app] Rustc git commit: 49cae55760da0a43428eba73abcb659bb70cf2e4
Prefab
======
PrefabEntity { parent: None, data: Some(Player { player: Named { name: "Zero" }, position: Position(1.0, 2.0, 3.0) }) }

Entities
========

| Entity                   | Handle<Prefab<Player>> | Parent | Position                | Player                 |
| ------------------------ | ---------------------- | ------ | ----------------------- | ---------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }       | None   | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } |
[INFO][amethyst::app] Engine is shutting down
```
