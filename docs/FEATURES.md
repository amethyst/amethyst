# Features

## Existing

* [x] modular structure while providing a quick solution to start your project (the main crate)
* [x] powered by the parallel ECS library [Specs]
* [x]  [gfx]-based rendering engine with high customizability
* [x] input abstraction for keyboard and mouse, bindings defined in config files
* [x] parallel asset loading with high extensibility and hot-reloading
* [x] vertex skinning and property animation
* [x] 3D audio with support for multiple emitters
* [x] sprite rendering and texture animation
* [x] basic UI support for text, text fields, buttons and images
* [x] scenes can be imported from [glTF] files
* [x] includes a simple state manager

## Planned

* [ ] networking
* [ ] scripting support
* [ ] defining scenes and prefabs with RON files
* [ ] gamepad support
* [ ] platform support: Android, iOS
* [ ] modular, composable and extensible editor that can be fully controlled by a REPL

[Specs]: https://github.com/slide-rs/specs
[gfx]: https://github.com/gfx-rs/gfx
[glTF]: https://www.khronos.org/gltf/
