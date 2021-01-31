## Prefab Adapter

Creates a `PrefabData` using the adapter pattern.

No window is created for this demo, instead you should see debugging information printed to the console similar to:

```log
[INFO][amethyst::app] Initializing Amethyst...
[INFO][amethyst::app] Version: 0.15.0
[INFO][amethyst::app] Platform: x86_64-apple-darwin
[INFO][amethyst::app] Amethyst git commit: f0543d5c987b227f7ec904226cb53a66c1ff4525
[INFO][amethyst::app] Rustc version: 1.44.0 Stable
[INFO][amethyst::app] Rustc git commit: 49cae55760da0a43428eba73abcb659bb70cf2e4
Prefab
======
PrefabEntity { parent: None, data: Some(Pos3f { x: 1.0, y: 2.0, z: 3.0 }) }
PrefabEntity { parent: None, data: Some(Pos3i { x: 4, y: 5, z: 6 }) }

Entities
========

| Entity                   | Handle<Prefab<PositionPrefab>> | Position                |
| ------------------------ | ------------------------------ | ----------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }               | Position(1.0, 2.0, 3.0) |
| Entity(1, Generation(1)) | None                           | Position(4.0, 5.0, 6.0) |
[INFO][amethyst::app] Engine is shutting down
```
