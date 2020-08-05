## Prefab Basic

Creates a trivial `PrefabData` and instantiates an entity using the `Prefab` system.

No window is created for this demo, instead you should see debugging information printed to the console similar to:

```
[INFO][amethyst::app] Initializing Amethyst...
[INFO][amethyst::app] Version: 0.15.0
[INFO][amethyst::app] Platform: x86_64-apple-darwin
[INFO][amethyst::app] Amethyst git commit: 80b864b30b6c9f4dd9109999ba3baf7e4f0d0489
[INFO][amethyst::app] Rustc version: 1.44.0 Stable
[INFO][amethyst::app] Rustc git commit: 49cae55760da0a43428eba73abcb659bb70cf2e4
Prefab
======
PrefabEntity { parent: None, data: Some(Position(1.0, 2.0, 3.0)) }

Entities
========

| Entity                   | Handle<Prefab<Position>> | Parent | Position                |
| ------------------------ | ------------------------ | ------ | ----------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }         | None   | Position(1.0, 2.0, 3.0) |
[INFO][amethyst::app] Engine is shutting down
```