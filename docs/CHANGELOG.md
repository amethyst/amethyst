# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## [Unreleased]

### Major breaking changes

* Systems needing initialization with world resources must go through a `SystemDesc` intermediate builder. ([#1780])

### Added

* `SystemDesc` proc macro derive to simplify defining `SystemDesc`s. ([#1780])
* `UiButtonData` is now exported from `amethyst_ui` and can be used for custom widgets. ([#1859])
* Add an audio subchapter to the pong chapter. ([#1842])
* Add `DispatcherOperation` to store dispatcher build logic, which can be executed lazily. ([#1870])
* `AmethystApplication` takes in `SystemDesc`s through `with_system_desc`. ([#1882])
* `AmethystApplication::with_thread_local_desc` takes in `RunNowDesc`. ([#1882])
* Add `NineSlice` support to `UiImage`. ([#1896])
* `RenderingBundle` for full manual control of the rendering pipeline via a custom `GraphCreator` ([#1839]).
* `CameraOrtho::new` takes in `CameraOrthoWorldCoordinates`, which can be set to custom dimensions. ([#1916])

### Changed

* All `-Builder` structs in amethyst_ui/prefab.rs are now called `-Data`. ([#1859])
* `AmethystApplication` takes in a `System` instead of a closure for `with_system`. ([#1882])
* `AmethystApplication::with_thread_local` constraint relaxed to `RunNow` (previously `System`). ([#1882])
* `SystemDesc` proc macro supports `#[system_desc(event_reader_id)]` to register event reader. ([#1883])
* `SystemDesc` proc macro supports `#[system_desc(flagged_storage_reader(Component))]`. ([#1886])
* Use `SystemDesc` derive to generate `SystemDesc` implementations for common case systems. ([#1887])
* `DispatcherOperation` stores system name and dependencies as `String`s. ([#1891])
* `TextureProcessor` renamed to `TextureProcessorSystem`. ([#1839])
* `MeshProcessor` renamed to `MeshProcessorSystem`. ([#1839])
* `AmethystApplication::with_setup` now takes in `FnOnce(&mut World) + Send + 'static`. ([#1912])
* `AmethystApplication::with_setup` runs the function before the dispatcher. ([#1912])

### Fixed

* `RenderingBundle` is registered last in all examples. ([#1881])

[#1780]: https://github.com/amethyst/amethyst/pull/1780
[#1859]: https://github.com/amethyst/amethyst/pull/1859
[#1842]: https://github.com/amethyst/amethyst/pull/1842
[#1870]: https://github.com/amethyst/amethyst/pull/1870
[#1881]: https://github.com/amethyst/amethyst/pull/1881
[#1882]: https://github.com/amethyst/amethyst/pull/1882
[#1883]: https://github.com/amethyst/amethyst/pull/1883
[#1886]: https://github.com/amethyst/amethyst/pull/1886
[#1887]: https://github.com/amethyst/amethyst/pull/1887
[#1891]: https://github.com/amethyst/amethyst/pull/1891
[#1896]: https://github.com/amethyst/amethyst/pull/1896
[#1839]: https://github.com/amethyst/amethyst/pull/1839
[#1912]: https://github.com/amethyst/amethyst/pull/1912
[#1916]: https://github.com/amethyst/amethyst/pull/1916

## [0.12.0] - 2019-07-30

### Breaking changes

* `Float` newtype removed, moved back to `f32` primitive for all values ([#1747])
* `TextureProcessor` and `MeshProcessor` systems are now separated from `RenderingSystem` ([#1772])

### Added

* Add a feature flag `sentry` to disable the sentry dependency. ([#1804]) ([#1825])
* Fixes and renames regression from ([#1442]) added back `position_from_world` as `screen_to_world`. Also added
`world_to_screen`. Also adds `Transform::copy_local_to_global()' for `debug_assertion` builds ([#1733])
* Add `add_rectangle`, `add_rotated_rectangle`, `add_box`, `add_rotated_box`, `add_circle`, `add_rotated_circle`,
`add_cylinder`, `add_rotated_cylinder` and `add_sphere` functions to `DebugLinesComponent`
and the corresponding draw functions to `DebugLines`, to draw simple shapes with debug lines. ([#1766])
* `InputEvent::AxisMoved` is sent upon button press / release. ([#1512], [#1797])
* `UiImage` is updated to allow for partial textures and sprites. ([#1809],[#1811])
* Added `RenderingBundle` with a rendering plugin system, making rendering setup easier ([#1772])
* Documentation for `Tint` component. ([#1802])

### Changed

* Splitted the `/resources` directory of amethyst projects into `/assets` and `/config`. ([#1806])
* Rename FPSCounter, FPSCounterBundle, FPSCounterSystem to FpsCounter, FpsCounterBundle, FpsCounterSystem. ([#1719])
* Add Tint component support for sprites. ([#1756])
* Remove remaining <N: RealField> type parameter on GameDataBuilder, add Debug derive to LoggerConfig ([#1758])
* Inverted mouse wheel scroll direction event. Now using winit's standard.  ([#1767])
* Add `load_from_data_async` to Asset Loader. ([#1753])
* Add `SerializableFormat` marker trait which is now needed to be implemented for all the formats that are supposed to be serialized. ([#1720])
* Make the GltfSceneOptions field of GltfSceneFormat public. ([#1791])
* Updated fluent to version 0.6. ([#1800])
 `InputEvent<T>` now takes in the `BindingTypes` as a type parameter. ([#1797])
* Use `crossbeam-queue` crate directly. ([#1822]) 

### Fixed

* Fix stack overflow on serializing `Box<dyn Format<_>>`. ([#1720])
* Fix the steps for enabling the nightly flag in the pong tutorial. ([#1805])
* Fix animation unwrap on missing animated component. ([#1773])
* Fix tangent generation in procedural shapes. ([#1807])

[#1512]: https://github.com/amethyst/amethyst/issues/1512
[#1719]: https://github.com/amethyst/amethyst/pull/1719
[#1720]: https://github.com/amethyst/amethyst/pull/1720
[#1733]: https://github.com/amethyst/amethyst/pull/1733
[#1747]: https://github.com/amethyst/amethyst/pull/1747
[#1753]: https://github.com/amethyst/amethyst/pull/1753
[#1756]: https://github.com/amethyst/amethyst/pull/1756
[#1758]: https://github.com/amethyst/amethyst/pull/1758
[#1766]: https://github.com/amethyst/amethyst/pull/1766
[#1767]: https://github.com/amethyst/amethyst/pull/1719
[#1772]: https://github.com/amethyst/amethyst/pull/1772
[#1773]: https://github.com/amethyst/amethyst/pull/1773
[#1791]: https://github.com/amethyst/amethyst/pull/1791
[#1797]: https://github.com/amethyst/amethyst/pull/1797
[#1800]: https://github.com/amethyst/amethyst/pull/1800
[#1802]: https://github.com/amethyst/amethyst/pull/1802
[#1804]: https://github.com/amethyst/amethyst/pull/1804
[#1805]: https://github.com/amethyst/amethyst/pull/1805
[#1807]: https://github.com/amethyst/amethyst/pull/1807
[#1809]: https://github.com/amethyst/amethyst/issues/1809
[#1811]: https://github.com/amethyst/amethyst/pull/1811
[#1822]: https://github.com/amethyst/amethyst/pull/1822
[#1825]: https://github.com/amethyst/amethyst/pull/1825

## [0.11.0] - 2019-06

### Added

* Introduce `application_dir` utility ([#1213])
* Derive `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` for `Flipped` component. ([#1237])
* A way to change the default `Source` using `set_default_source` and `with_default_source`. ([#1256])
* "How To" guides for using assets and defining custom assets. ([#1251])
* Explanation on how prefabs function in Amethyst. ([#1114])
* `amethyst_renderer::Rgba` is now a `Component` that changes the color and transparency of the entity
it is attached to. ([#1282])
* `AutoFov` and `AutoFovSystem` to adjust horizontal FOV to screen aspect ratio. ([#1281])
* Add `icon` to `DisplayConfig` to set a window icon using a path to a file ([#1373])
* Added setting to control gfx_device_gl logging level separately, and set it to Warn by default. ([#1404])
* Add `loaded_icon` to `DisplayConfig` to set a window icon programatically ([#1405])
* Added optional feature gates which will reduce compilation times when used. ([#1412])
* Several passes got `with_transparency_settings` which changes the transparency settings for the pass. ([#1419])
* Add `SpriteRenderPrefab`. ([#1435])
* Add `ScreenSpace` component. Draws entities using the screen coordinates. ([#1424])
* Add `add_removal_to_entity` function. ([#1445])
* Add `position_from_screen` to `Camera`. Transforms position from screen space to camera space. ([#1442])
* Add `SpriteScenePrefab`. Allows load sprites from a grid and add them to the `SpriteRenderer`. ([#1469])
* Add `Widgets` resource. Allows keeping track of UI entities and their components and iterating over them. ([#1390])
* `AmethystApplication` takes in application name using `with_app_name(..)`. ([#1499])
* Add `NetEvent::Reliable` variant. When added to NetConnection, these events will eventually reach the target. ([#1513])
* "How To" guides for defining state-specific dispatchers. ([#1498])
* Adding support for AMETHYST_NUM_THREADS environment variable to control size of the threads pool used by thread_pool_builder.
* Add `Input` variant to `StateEvent`. ([#1478])
* Support type parameters in `EventReader` derive. ([#1478])
* Derive `Debug`, `PartialEq`, `Eq` for `Source`. ([#1591])
* Added `events` example which demonstrates working even reader and writer in action. ([#1538])
*  Implement builder like functionality for `AnimationSet` and `AnimationControlSet` ([#1568])
* Add `get_mouse_button` and `is_mouse_button_down` utility functions to amethyst_input. ([#1582])
* Add `amethyst_input::Axis::MouseWheel` ([#1642])
* Add `amethyst_input::BindingError::MouseWheelAlreadyBound` ([#1642])
* Add `amethyst_input::InputHandler::send_frame_begin` ([#1642])
* Add `amethyst_input::InputHandler::mouse_wheel_value` ([#1642])
* Added `Float::from_f32` and `Float::from_f64` `const fn`s so `Float` can be used as `const`. ([#1687])
* Add `debug_lines_ortho` example. ([#1703])

### Changed

* `#[derive(PrefabData)]` now supports enums as well as structs
* Make `frame_limiter::do_sleep` calculate the amount of time to sleep instead of calling `sleep(0)` ([#1446])
* Make `application_root_dir` return a `Result<Path>` instead of a `String` ([#1213])
* Remove unnecessary texture coordinates offset in `Sprite::from_pixel_values` ([#1267])
* Changed `ActiveCamera` to have the `Option` inside. ([#1280])
* `AudioBundle::new()` no longer exists, as `AudioBundle` is now a unit type. It also no longer initializes the `DjSystem` ([#1356])
* Convert everything to use err-derive and amethyst_error ([#1365])
* Removed redundant code in `renderer.rs` ([#1375])
* Refactored audio initialization to be more bundle-centric ([#1388])
* Changed argument types of `exec_removal` to allow use of both Read and Write Storages. ([#1397])
* Changed default log level to Info. ([#1404])
* Remove unnecessary `mut` from `AnimationControlSet::has_animation` ([#1408])
* Moved amethyst_gltf from development workspace to be like the other amethyst_* subcrates. ([#1411])
* Re-exported amethyst_gltf by amethyst as amethyst::gltf. ([#1411])
* `Default::default` now returns a pass with transparency enabled for all applicable passes. ([#1419])
* Several passes had a function named `with_transparency` changed to accept a boolean. ([#1419])
* `FrameRateLimitConfig` has a `new` constructor, and its fields are made public. ([#1436])
* Derive `Deserialize, Serialize` for `MaterialPrimitive` and `SpriteRenderPrimitive`, remove
extra bounds from `AnimatablePrefab` and `AnimationSetPrefab` ([#1435])
* Renamed `amethyst_core::specs` to `amethyst_core::ecs` and `amethyst_core::nalgebra` to `amethyst_core::math`. ([#1410])
* Simplified some of the conditionals in the Pong tutorial. ([#1439])
* Changed the names of many Transform functions to better reflect their actual function and reduce potential semantic confusion ([#1451])
* `ProgressCounter#num_loading()` no longer includes failed assets. ([#1452])
* `SpriteSheetFormat` field renamed from `spritesheet_*` to `texture_*`. ([#1469])
* Add new `keep_aspect_ratio` field to `Stretch::XY`. ([#1480])
* Renamed `Text` UI Prefab to `Label` in preparation for full widget integration in prefabs. ([#1390])
* `amethyst_test` includes the application name of a failing test. ([#1499])
* `amethyst_test` returns the panic message of a failed execution. ([#1499])
* Rename `NetEvent::Custom` variant to `NetEvent::Unreliable`. ([#1513])
* Updated laminar to 0.2.0. ([#1502])
* Large binary files in examples are now tracked with `git-lfs`. ([#1509])
* Allowed the user to arrange with laminar. ([#1523])
* Removed `NetEvent::Custom` and added `NetEvent::Packet(NetPacket)` ([#1523])
* Fixed update is no longer frame rate dependent ([#1516])
* Display the syntax error when failing to parse sprite sheets  ([#1526])
* Added generic parameter type to `Transform` to configure floating point precision (then removed). ([#1334]) ([#1584])
* `NetConnection` is automatically created when client starts sends data to server. ([#1539])
* User will receive `NetEvent::Connected` on new connection and `NetEvent::Disconnected` on disconnect. ([#1539])
* Added a `pivot` field to `UiTransform`. ([#1571])
* Fix fly_camera example initial camera and cube position. ([#1582])
* Add to fly_camera example code to release and capture back mouse input, and to show and hide cursor. ([#1582])
* Updated `rodio` to `0.9`. ([#1683])

#### Rendy support

* Brand new way to define rendering pipelines.
* OpenGL support temporarily dropped, Vulkan and Metal support added.
* Normalized texel coordinates are now in Vulkan convention (top-left 0.0, bottom-right 1.0), mirrored vertically compared to old one.
* World space is now Y-up consistently for all projections (2D and 3D).
* `Format` type no longer has associated `Options` and is now object-safe. It is expected to carry required options itself.
* `Format` now supports tag-based deserialization, it is no longer required to provide specific format to prefab type.
* Combined input axis/action generics into single type.
* `Material` is now an asset. Must be turned into handle before putting on an entity.
* Removed `Flipped` component. Use `flip_horizontal` and `flip_vertical` sprite property instead.
* Added [Rendy migration guide][rendy_migration]. ([#1626])

### Removed

- Removed all `NetEvent's` because they were not used. ([#1539])
- Removed filter logic, because it didn't do anything, will be added back in a later version (NetFilter, FilterConnected). ([#1539])

### Fixed

* Optimize loading of wavefront obj mesh assets by getting rid of unnecessary allocations. ([#1454])
* Fixed the "json" feature for amethyst_assets. ([#1302])
* Fixed default system font loading to accept uppercase extension ("TTF"). ([#1328])
* Set width and height of Pong Paddles ([#1363])
* Fix omission in `PosNormTangTex` documentation. ([#1371])
* Fix division by zero in vertex data building ([#1481])
* Fix tuple index generation on `PrefabData` and `EventReader` proc macros. ([#1501])
* Avoid segmentation fault on Windows when using `AudioBundle` in `amethyst_test`. ([#1595], [#1599])

[rendy_migration]: https://book.amethyst.rs/master/appendices/b_migration_notes/rendy_migration.html
[#1114]: https://github.com/amethyst/amethyst/pull/1114
[#1213]: https://github.com/amethyst/amethyst/pull/1213
[#1237]: https://github.com/amethyst/amethyst/pull/1237
[#1251]: https://github.com/amethyst/amethyst/pull/1251
[#1256]: https://github.com/amethyst/amethyst/pull/1256
[#1267]: https://github.com/amethyst/amethyst/pull/1267
[#1280]: https://github.com/amethyst/amethyst/pull/1280
[#1282]: https://github.com/amethyst/amethyst/pull/1282
[#1281]: https://github.com/amethyst/amethyst/pull/1281
[#1302]: https://github.com/amethyst/amethyst/pull/1302
[#1328]: https://github.com/amethyst/amethyst/pull/1328
[#1334]: https://github.com/amethyst/amethyst/pull/1334
[#1356]: https://github.com/amethyst/amethyst/pull/1356
[#1363]: https://github.com/amethyst/amethyst/pull/1363
[#1365]: https://github.com/amethyst/amethyst/pull/1365
[#1371]: https://github.com/amethyst/amethyst/pull/1371
[#1373]: https://github.com/amethyst/amethyst/pull/1373
[#1375]: https://github.com/amethyst/amethyst/pull/1375
[#1388]: https://github.com/amethyst/amethyst/pull/1388
[#1390]: https://github.com/amethyst/amethyst/pull/1390
[#1397]: https://github.com/amethyst/amethyst/pull/1397
[#1404]: https://github.com/amethyst/amethyst/pull/1404
[#1408]: https://github.com/amethyst/amethyst/pull/1408
[#1405]: https://github.com/amethyst/amethyst/pull/1405
[#1411]: https://github.com/amethyst/amethyst/pull/1411
[#1412]: https://github.com/amethyst/amethyst/pull/1412
[#1419]: https://github.com/amethyst/amethyst/pull/1419
[#1424]: https://github.com/amethyst/amethyst/pull/1424
[#1435]: https://github.com/amethyst/amethyst/pull/1435
[#1436]: https://github.com/amethyst/amethyst/pull/1436
[#1410]: https://github.com/amethyst/amethyst/pull/1410
[#1439]: https://github.com/amethyst/amethyst/pull/1439
[#1445]: https://github.com/amethyst/amethyst/pull/1445
[#1446]: https://github.com/amethyst/amethyst/pull/1446
[#1451]: https://github.com/amethyst/amethyst/pull/1451
[#1452]: https://github.com/amethyst/amethyst/pull/1452
[#1454]: https://github.com/amethyst/amethyst/pull/1454
[#1442]: https://github.com/amethyst/amethyst/pull/1442
[#1469]: https://github.com/amethyst/amethyst/pull/1469
[#1478]: https://github.com/amethyst/amethyst/pull/1478
[#1481]: https://github.com/amethyst/amethyst/pull/1481
[#1480]: https://github.com/amethyst/amethyst/pull/1480
[#1498]: https://github.com/amethyst/amethyst/pull/1498
[#1499]: https://github.com/amethyst/amethyst/pull/1499
[#1501]: https://github.com/amethyst/amethyst/pull/1501
[#1502]: https://github.com/amethyst/amethyst/pull/1515
[#1513]: https://github.com/amethyst/amethyst/pull/1513
[#1509]: https://github.com/amethyst/amethyst/pull/1509
[#1523]: https://github.com/amethyst/amethyst/pull/1523
[#1524]: https://github.com/amethyst/amethyst/pull/1524
[#1526]: https://github.com/amethyst/amethyst/pull/1526
[#1538]: https://github.com/amethyst/amethyst/pull/1538
[#1539]: https://github.com/amethyst/amethyst/pull/1543
[#1568]: https://github.com/amethyst/amethyst/pull/1568
[#1571]: https://github.com/amethyst/amethyst/pull/1571
[#1584]: https://github.com/amethyst/amethyst/pull/1584
[#1591]: https://github.com/amethyst/amethyst/pull/1591
[#1582]: https://github.com/amethyst/amethyst/pull/1582
[#1595]: https://github.com/amethyst/amethyst/issues/1595
[#1599]: https://github.com/amethyst/amethyst/pull/1599
[#1626]: https://github.com/amethyst/amethyst/pull/1626
[#1642]: https://github.com/amethyst/amethyst/pull/1642
[#1683]: https://github.com/amethyst/amethyst/pull/1683
[#1687]: https://github.com/amethyst/amethyst/pull/1687
[#1703]: https://github.com/amethyst/amethyst/pull/1703

## [0.10.0] - 2018-12

### Added

* Derive `PrefabData` for `CameraOrtho` component ([#1188])
* Partially migrate the project to Rust 2018.  Full migration will be completed at some point after 2019-01-31 ([#1098])
* `SystemExt::pausable` for better ergonomics when pausing systems for specific states ([#1146]).
* `amethyst_test` test framework for ergonomic testing of Amethyst applications ([#1000])
* combinations of buttons triggering actions ([#1043])
* `UiPrefab` field `hidden: bool` to hide entities ([#1051])
* `PrefabData` can now be derived for many situations, see the book for more information ([#1035])
* Support for DirectionalLight and SpotLight in PBM pass. ([#1074], [#1081])
* `UiWidget` variant `Custom` for custom composited widgets ([#1112])
* `AssetLoaderSystemData` abstracts resources needed from `World` to do asset loading ([#1090])
* `amethyst_ui::get_default_font` supports loading system font from Path. ([#1108])
* Added render utilities to easily create `Material` and `Handle<Texture>`. ([#1126])
* Added `Callback` and `CallbackQueue` for use in asynchronous contexts. ([#1125])
* Added Trans event queue. Used to trigger state transitions from systems. Also used to trigger multiple state transitions at once. (For example, to `Trans::Pop` two states.) ([#1069])
* `sprite_camera_follow` example showing how to use a Camera that has a sprite Parent ([#1099])
* Added capabilities for the `DrawFlat2D` pass to draw `TextureHandle`s by themselves. Also added a simple example for this. ([#1153])
* Added a `Flipped` component which allows flipping sprites or images horizontally and vertically. ([#1153])
* Added transform constructor function `Transform::new()`. ([#1187])
* Implement generic `EventRetriggerSystem`, which enables dispatching new events as a reaction to other events ([#1189])

### Changed

* Minimum Rust version is now `1.31.0` &ndash; Rust 2018. ([#1224])
* `Transform::look_at` renamed to `Transform::face_towards` and behavior fixed. ([#1142])
* `Material` animations now directly use `Handle<Texture>` instead of using indirection. ([#1089])
* `SpriteRenderPrimitive::SpriteSheet` now takes `Handle<SpriteSheet>` instead of a `u64` ID. ([#1089])
* `nalgebra` is now the math library used by the engine. ([#1066])
* The `amethyst::renderer::Projection::orthographic` function has had its parameter order changed to match that of `nalgebra` ([#1066])
* `SpriteSheet` now use `TextureHandle` directly instead of a `u64` ID coupled with `MaterialTextureSet`. ([#1117])
* Updated `specs` to `0.14` and `specs-hierarchy` to `0.3`. ([#1122])
* Updated `winit` to `0.18` (see [Winit's changelog][winit_018]). ([#1131])
* Updated `glutin` to `0.19` (see [Glutin's changelog][glutin_019]). ([#1131])
* Renamed the `DrawSprite` pass to `DrawFlat2D` as it now handles both sprites and images without spritesheets. ([#1153])
* `BasicScenePrefab` deserialization now returns an error on invalid fields. ([#1164])
* Reordered arguments for `Transform::set_rotation_euler` to match nalgebra's Euler angles. ([#1052])
* Remove lifetimes from `SimpleState` ([#1198])
* Button interactions are now handled through an `EventRetriggerSystem`, specifically hover/click sounds and image/color changes ([#1189])

### Removed

* `SpriteSheetSet` is removed as it is no longer needed. ([#1089])
* `MaterialTextureSet` is removed as it is no longer needed. ([#1117])
* `amethyst::core::Orientation` has been removed because of limited use. ([#1066])
* `TimedDestroySystem` has been split into `DestroyAtTimeSystem` and `DestroyInTimeSystem`. ([#1129])
* Reverted [MacOS OpenGL workaround][#972] in favor of the upstream fix in `glutin`. ([#1184])
* `OnUiActionImage` and `OnUiActionSound` have been removed as they now work through `EventRetrigger`s ([#1189])

### Fixed

* `SpriteSheetFormat` converts pixel coordinates to texture coordinates on load. ([#1181])

[#1146]: https://github.com/amethyst/amethyst/pull/1146
[#1144]: https://github.com/amethyst/amethyst/pull/1144
[#1000]: https://github.com/amethyst/amethyst/pull/1000
[#1043]: https://github.com/amethyst/amethyst/pull/1043
[#1051]: https://github.com/amethyst/amethyst/pull/1051
[#1035]: https://github.com/amethyst/amethyst/pull/1035
[#1069]: https://github.com/amethyst/amethyst/pull/1069
[#1074]: https://github.com/amethyst/amethyst/pull/1074
[#1081]: https://github.com/amethyst/amethyst/pull/1081
[#1090]: https://github.com/amethyst/amethyst/pull/1090
[#1112]: https://github.com/amethyst/amethyst/pull/1112
[#1089]: https://github.com/amethyst/amethyst/pull/1089
[#1098]: https://github.com/amethyst/amethyst/pull/1098
[#1099]: https://github.com/amethyst/amethyst/pull/1099
[#1108]: https://github.com/amethyst/amethyst/pull/1108
[#1126]: https://github.com/amethyst/amethyst/pull/1126
[#1125]: https://github.com/amethyst/amethyst/pull/1125
[#1066]: https://github.com/amethyst/amethyst/pull/1066
[#1117]: https://github.com/amethyst/amethyst/pull/1117
[#1122]: https://github.com/amethyst/amethyst/pull/1122
[#1129]: https://github.com/amethyst/amethyst/pull/1129
[#1131]: https://github.com/amethyst/amethyst/pull/1131
[#1153]: https://github.com/amethyst/amethyst/pull/1153
[#1164]: https://github.com/amethyst/amethyst/pull/1164
[#1142]: https://github.com/amethyst/amethyst/pull/1142
[#1052]: https://github.com/amethyst/amethyst/pull/1052
[#1181]: https://github.com/amethyst/amethyst/pull/1181
[#1184]: https://github.com/amethyst/amethyst/pull/1184
[#1187]: https://github.com/amethyst/amethyst/pull/1187
[#1188]: https://github.com/amethyst/amethyst/pull/1188
[#1198]: https://github.com/amethyst/amethyst/pull/1198
[#1224]: https://github.com/amethyst/amethyst/pull/1224
[#1189]: https://github.com/amethyst/amethyst/pull/1189
[winit_018]: https://github.com/tomaka/winit/blob/v0.18.0/CHANGELOG.md#version-0180-2018-11-07
[glutin_019]: https://github.com/tomaka/glutin/blob/master/CHANGELOG.md#version-0190-2018-11-09

## [0.9.0] - 2018-10
### Added
* Added base networking implementation and the `amethyst_network` crate. ([#969])
* Support for debug lines using `DebugLines` pass, and `DebugLines` component or resource. ([#917], [#957])
* Added JsonFormat ([#950]).
* `SpriteRender` pass to draw sprites without using `Material` and `Mesh`. ([#829], [#830])
* Sprite animation uses the `SpriteRenderChannel`. ([#829], [#830])
* State::handle_event can now handle multiple types of events. ([#887])
* Added Named Component. ([#879])([#896])
* Support for progressive jpeg loading. ([#877])
* New `application_root_dir()` function in `amethyst_utils`. ([#831])
* Load node names for glTF prefabs. ([#905])
* Added automatic camera matrix resizing to allow clean screen resizes. ([#920])
* Added the Removal component to facilitate manual entity removal and scene cleaning. ([#920])
* Added DestroyAtTime and DestroyInTime components to easily destroy entities. ([#920])
* Support for loading TGA images. ([#934])
* GltfPrefab adds MeshData as a component on loaded entities. This is not configurable until the Prefab rework. ([#946])
* Added implementation of From<Vector3<f32>> for Transform which creates a Transform using Vector3 as the translation vector. ([#946])
* New vertices() method on MeshCreator trait. ([#946])
* Support for text alignment (align left, center, right). ([#965])
* Support for multiline text. ([#965])
* Added custom aspect ratio to OrthoCamera. ([#983])
* Added AntiStorage documentation to the book. ([#997])
* You can now stop the rotation of the FreeRotationSystem by setting HideCursor.hide value to false. ([#997])
* Support for logging to file, toggle for logging to stdout. ([#976], [#994])
* Added a `Hidden` Component, that hides a single entity, and a HideHierarchySystem that toggles `Hidden` on all children when used. ([#1001])
* Documentation for drawing sprites. ([#971])
* Added `shadow_update()` and `shadow_fixed_update()` to the `State` trait. ([#1006])
* Added configurable width for debug lines. ([#1016])
* Added `TextureMetadata::srgb_scale()` for default texture metadata with nearest filter. ([#1023])
* Added motivation to use Amethyst over gluing the building blocks yourself in the book. ([#1057])
* Added `Config::load_bytes` for reading configuration from raw bytes. ([#1067])

### Changed

* Sprites contain their dimensions and offsets to render them with the right size and desired position. ([#829], [#830])
* Texture coordinates for sprites are 1.0 at the top of the texture and 0.0 at the bottom. ([#829], [#830])
* Made get_camera public. ([#878])
* Simplified creating states with SimpleState and EmptyState. ([#887])
* Updated ProgressCounter to show loading errors. ([#892])
* Replaced the `imagefmt` crate with `image`. ([#877])
* Optimize Sprite rendering via batching. ([#902])
* Derive `Debug` and `PartialEq` for `amethyst_input::Axis`. ([#903], [#904])
* Updated `winit` to `0.17` (see [Winit's changelog][winit_017]). ([#906])
* Updated `glutin` to `0.18` (see [Glutin's changelog][glutin_018]). ([#906])
* Updated `gfx_window_glutin` to `0.26`. ([#906])
* Updated `hetseq` to `0.2`. ([#906])
* Removed unwraps from StateMachine ([#940])
* Renamed ArcBallMovementSystem to ArcBallRotationSystem. ([#946])
* Moved the ArcBallMovementSystem::get_axis method to amethyst_input/src/utils: get_input_axis_simple ([#946])
* Ui Y axis is now from bottom to top. ([#946])
* Fixed issue with global anchors not actually aligning ui elements and containers properly. ([#946])
* Fixed issue with ui events not triggering at times. ([#946])
* Reduced the complexity of the UiPass and associated shaders. ([#946])
* Added comments to UiPass and shaders explaining what is going on. ([#946])
* The z in UiTransformBuilder now defaults to 1 instead of 0, allowing to skip defining the z in the ui prefabs. ([#946])
* Added comments to ui prefab. ([#946])
* Summarized all `use amethyst::` statements to allow collapsing in IDE's. ([#974])
* `Application` now uses `EventReader`s to determine what events to send to the `State`s, more information in the `State`
  book chapter ([#996])
* Breaking: Refactor `TextureMetadata` so filter method and clamping can be configured more easily ([#981])
* Renamed `PrefabData` functions to be easier to understand ([#1008])

### Removed

* `LMenu` and `RMenu` key codes, following the `winit` update. ([#906])

### Fixed

* Material ids in GLTF loader caused multiple GLTF files to get incorrect materials applied. ([#915])
* Fix render gamma for most textures. ([#868])
* Joint entities can only be part of a single skin: Materials are not swapped anymore. ([#933])
* Fixed regression in sprite positioning after batching. ([#929])
* Now loading default fonts from the system for UiButton ([#964])
* Fixed single frame animation ([#1015])
* Improved compatibility with older drivers ([#1012])
* Forgotten `channel` field on `examples/ui` prefab ([#1024])
* `AssetPrefab` loaded files at an incorrect time ([#1020])
* Removed unreachable code in `TexturePrefab` ([#1020])
* Fix OpenGL not rendering on window creation due to `glutin` bug ([#972])
* Fix debug lines panic when no lines are rendered ([#1049])

[#829]: https://github.com/amethyst/amethyst/issues/829
[#830]: https://github.com/amethyst/amethyst/pull/830
[#879]: https://github.com/amethyst/amethyst/pull/879
[#878]: https://github.com/amethyst/amethyst/pull/878
[#887]: https://github.com/amethyst/amethyst/pull/887
[#892]: https://github.com/amethyst/amethyst/pull/892
[#877]: https://github.com/amethyst/amethyst/pull/877
[#878]: https://github.com/amethyst/amethyst/pull/878
[#896]: https://github.com/amethyst/amethyst/pull/896
[#831]: https://github.com/amethyst/amethyst/pull/831
[#902]: https://github.com/amethyst/amethyst/pull/902
[#905]: https://github.com/amethyst/amethyst/pull/905
[#920]: https://github.com/amethyst/amethyst/pull/920
[#903]: https://github.com/amethyst/amethyst/issues/903
[#904]: https://github.com/amethyst/amethyst/pull/904
[#906]: https://github.com/amethyst/amethyst/pull/906
[#915]: https://github.com/amethyst/amethyst/pull/915
[#868]: https://github.com/amethyst/amethyst/pull/868
[#917]: https://github.com/amethyst/amethyst/issues/917
[#933]: https://github.com/amethyst/amethyst/pull/933
[#929]: https://github.com/amethyst/amethyst/pull/929
[#934]: https://github.com/amethyst/amethyst/pull/934
[#940]: https://github.com/amethyst/amethyst/pull/940
[#946]: https://github.com/amethyst/amethyst/pull/946
[#950]: https://github.com/amethyst/amethyst/pull/950
[#957]: https://github.com/amethyst/amethyst/pull/957
[#964]: https://github.com/amethyst/amethyst/pull/964
[#965]: https://github.com/amethyst/amethyst/pull/965
[#969]: https://github.com/amethyst/amethyst/pull/969
[#983]: https://github.com/amethyst/amethyst/pull/983
[#971]: https://github.com/amethyst/amethyst/pull/971
[#972]: https://github.com/amethyst/amethyst/issue/972
[#974]: https://github.com/amethyst/amethyst/pull/974
[#976]: https://github.com/amethyst/amethyst/pull/976
[#981]: https://github.com/amethyst/amethyst/pull/981
[#994]: https://github.com/amethyst/amethyst/pull/994
[#996]: https://github.com/amethyst/amethyst/pull/996
[#997]: https://github.com/amethyst/amethyst/pull/997
[#1001]: https://github.com/amethyst/amethyst/pull/1001
[#1006]: https://github.com/amethyst/amethyst/pull/1006
[#1008]: https://github.com/amethyst/amethyst/pull/1008
[#1012]: https://github.com/amethyst/amethyst/pull/1012
[#1015]: https://github.com/amethyst/amethyst/pull/1015
[#1016]: https://github.com/amethyst/amethyst/pull/1016
[#1024]: https://github.com/amethyst/amethyst/pull/1024
[#1020]: https://github.com/amethyst/amethyst/pull/1020
[#1023]: https://github.com/amethyst/amethyst/pull/1023
[#1057]: https://github.com/amethyst/amethyst/pull/1057
[#1049]: https://github.com/amethyst/amethyst/pull/1049
[#1067]: https://github.com/amethyst/amethyst/pull/1067
[winit_017]: https://github.com/tomaka/winit/blob/master/CHANGELOG.md#version-0172-2018-08-19
[glutin_018]: https://github.com/tomaka/glutin/blob/master/CHANGELOG.md#version-0180-2018-08-03

## [0.8.0] - 2018-08

### Added

* UI `ScaleMode` is now functional, permitting percentage based `UiTransform`s. ([#774])
* Add serde trait derives to many core components ([#760])
* Add a generic asset `Format` for `ron` files ([#760])
* Improve error handling for asset loading ([#773])
* Add bundle for the arc ball camera ([#770])
* Add utility functions for dealing with common input ([#759])
* Add alpha cutoff support to the PBR shader ([#756])
* Basic renderer setup helper function ([#771])
* Shape mesh generators ([#777])
* Derive `PartialEq` for `SpriteSheet` ([#789])
* Add core support for Prefabs ([#716])
* Add shape prefab support ([#785])
* Specialised UI prefab format ([#786])
* Add generation of normals/tangents in GLTF ([#784])
* Localisation using FTL files and the fluent-rs library ([#663])
* Add basic scene prefab ([#791])
* Improve ergonomics of examples ([#793])
* Beginner-friendly utilities for sprite rendering ([#804])
* Derive `PartialEq` for `MaterialPrimitive` ([#809])
* Make `with_bindings_from_file` return a Result ([#811])
* Logger initialization is now optional and can be enabled with a call to `amethyst::start_logger()` ([#815])
* Gamepad support with optional builtin SDL controller event source ([#818])
* Promote `UiButton` to a fundamental Ui component ([#798])

### Changed

* UI systems will now never overwrite your local `UiTransform` values ([#774])
* Global `UiTransform` values are no longer writable ([#774])
* `UiResize` refactored to be more user friendly and more helpful ([#774])
* `Anchored` and `Stretched` components have been folded into `UiTransform` ([#774])
* Refactored asset loading so `Processor`s can defer storage insertion ([#760])
* Moved `MaterialTextureSet` to the renderer crate ([#760])
* Use `fresnel` function in PBR shader ([#772])
* Remove boilerplate for `run` + `main` in examples ([#764])
* Update dependencies ([#752], [#751], [#817])
* Formalized and documented support for overriding the global logger ([#776])
* Refactor GLTF loader to use prefabs ([#784])
* Point lights use `GlobalTransform` for positioning rather than a separate `center` ([#794])
* Point lights now require a `GlobalTransform` component to be included in rendering ([#794])
* `amethyst_input::input_handler::{keys_that_are_down, mouse_buttons_that_are_down, scan_codes_that_are_down, buttons_that_are_down}` now all return `impl Iterator` instead of concrete wrapper types ([#816])
* Renamed is_key to is_key_down and fixed example to react when the key is pressed instead of released. ([#822])
* SpriteRenderData now allows to retrieve the MeshHandle and Material before inserting them into an entity. ([#825])
* Update the pong tutorial + changelog for SpriteRenderData. ([#805])
* Loosen up generic type bounds for InputBundle. ([#808])

### Removed

* Remove `amethyst_input::{KeyCodes, ScanCodes, MouseButtons, Buttons}` in favor of `impl trait` ([#816])

### Fixed

* Resizing fixed on OSX ([#767])
* Fix color format ([#766])
* Remove individual example READMEs ([#758])
* Log an error if a pass tries to render a mesh with incompatible vertex buffers ([#749])
* Standardize vsync across examples ([#746])
* Minor Pong tutorial fixes. ([#807])
* Fix wrong resource paths in examples. ([#812])

[#663]: https://github.com/amethyst/amethyst/pull/663
[#746]: https://github.com/amethyst/amethyst/pull/746
[#749]: https://github.com/amethyst/amethyst/pull/749
[#751]: https://github.com/amethyst/amethyst/pull/751
[#752]: https://github.com/amethyst/amethyst/pull/752
[#756]: https://github.com/amethyst/amethyst/pull/756
[#758]: https://github.com/amethyst/amethyst/pull/758
[#759]: https://github.com/amethyst/amethyst/pull/759
[#760]: https://github.com/amethyst/amethyst/pull/760
[#764]: https://github.com/amethyst/amethyst/pull/764
[#766]: https://github.com/amethyst/amethyst/pull/766
[#767]: https://github.com/amethyst/amethyst/pull/767
[#770]: https://github.com/amethyst/amethyst/pull/770
[#771]: https://github.com/amethyst/amethyst/pull/771
[#772]: https://github.com/amethyst/amethyst/pull/772
[#773]: https://github.com/amethyst/amethyst/pull/773
[#774]: https://github.com/amethyst/amethyst/pull/774
[#777]: https://github.com/amethyst/amethyst/pull/777
[#776]: https://github.com/amethyst/amethyst/pull/776
[#798]: https://github.com/amethyst/amethyst/pull/798
[#716]: https://github.com/amethyst/amethyst/pull/716
[#784]: https://github.com/amethyst/amethyst/pull/784
[#785]: https://github.com/amethyst/amethyst/pull/785
[#786]: https://github.com/amethyst/amethyst/pull/786
[#791]: https://github.com/amethyst/amethyst/pull/791
[#789]: https://github.com/amethyst/amethyst/pull/789
[#793]: https://github.com/amethyst/amethyst/pull/793
[#804]: https://github.com/amethyst/amethyst/pull/804
[#805]: https://github.com/amethyst/amethyst/pull/805
[#807]: https://github.com/amethyst/amethyst/pull/807
[#808]: https://github.com/amethyst/amethyst/pull/808
[#809]: https://github.com/amethyst/amethyst/pull/809
[#811]: https://github.com/amethyst/amethyst/pull/811
[#794]: https://github.com/amethyst/amethyst/pull/794
[#812]: https://github.com/amethyst/amethyst/pull/812
[#816]: https://github.com/amethyst/amethyst/pull/816
[#815]: https://github.com/amethyst/amethyst/pull/815
[#817]: https://github.com/amethyst/amethyst/pull/817
[#818]: https://github.com/amethyst/amethyst/pull/818
[#822]: https://github.com/amethyst/amethyst/pull/822
[#825]: https://github.com/amethyst/amethyst/pull/825

## [0.7.0] - 2018-05

### Added

* Documentation for Animation crate ([#631]).
* Support for rendering sprites ([#638]).
* Fly Camera ([#578]).
* UI Layouts ([#591]).
* UI Events ([#580]).
* Introduce a generic animation system, with support for both  transform and texture animation ([#558]), ([#566]), ([#567]), ([#569]), ([#570]), ([#611]), ([#641]), ([#644])
* Add transparency support to core passes ([#543]), ([#574]), ([#584])
* Add vertex skinning ([#545]), ([#619])
* Expose a basic visibility ordering system, with the ability to swap in better replacement systems ([#595])
* Audio `Output` is now added directly rather than as an `Option`, should now be fetched with `Option<Read<'a, Output>>` ([#679])
* New nightly feature that enables `shred`s nightly feature ([#689])
* `Transform` refactored, and added lots of utility functions ([#660])
* Add new raw mouse events for use with camera rotation ([#699])
* Add UiButtons and UiButtonBuilder ([#613])
* Add arc ball camera ([#700])

### Changed

* Update dependencies to the newest versions: cgmath, winit, glutin, gfx, gfx_glyph ([#527]), ([#572]), ([#648])
* Rodio updated to 0.7 ([#676])
* Refactored bundles to only contain `System`s ([#675])
* Refactor to use new specs, major breakage! ([#674]), ([#679]), ([#683]), ([#662]).
* Upgrade to winit 1.13.1 ([#698])
* Refactor game data, permit greater extensibility ([#691])
* Disable multisampling on all examples, and add a single example with multisampling on ([#671])

### Fixed

* Asset loading tolerates paths constructed using back slashes ([#623]).
* Pong text alignment ([#621]).
* Updated book introduction ([#588]).
* Renderable runtime crash ([#586]).

[#580]: https://github.com/amethyst/amethyst/pull/580
[#591]: https://github.com/amethyst/amethyst/pull/591
[#578]: https://github.com/amethyst/amethyst/pull/578
[#586]: https://github.com/amethyst/amethyst/pull/586
[#588]: https://github.com/amethyst/amethyst/pull/588
[#631]: https://github.com/amethyst/amethyst/pull/631
[#638]: https://github.com/amethyst/amethyst/pull/638
[#623]: https://github.com/amethyst/amethyst/pull/623
[#621]: https://github.com/amethyst/amethyst/pull/621
[#558]: https://github.com/amethyst/amethyst/pull/558
[#566]: https://github.com/amethyst/amethyst/pull/566
[#567]: https://github.com/amethyst/amethyst/pull/567
[#569]: https://github.com/amethyst/amethyst/pull/569
[#570]: https://github.com/amethyst/amethyst/pull/570
[#611]: https://github.com/amethyst/amethyst/pull/611
[#641]: https://github.com/amethyst/amethyst/pull/641
[#644]: https://github.com/amethyst/amethyst/pull/644
[#543]: https://github.com/amethyst/amethyst/pull/543
[#574]: https://github.com/amethyst/amethyst/pull/574
[#584]: https://github.com/amethyst/amethyst/pull/584
[#545]: https://github.com/amethyst/amethyst/pull/545
[#619]: https://github.com/amethyst/amethyst/pull/619
[#527]: https://github.com/amethyst/amethyst/pull/527
[#572]: https://github.com/amethyst/amethyst/pull/572
[#648]: https://github.com/amethyst/amethyst/pull/648
[#595]: https://github.com/amethyst/amethyst/pull/595
[#679]: https://github.com/amethyst/amethyst/pull/679
[#675]: https://github.com/amethyst/amethyst/pull/675
[#676]: https://github.com/amethyst/amethyst/pull/676
[#674]: https://github.com/amethyst/amethyst/pull/674
[#679]: https://github.com/amethyst/amethyst/pull/679
[#683]: https://github.com/amethyst/amethyst/pull/683
[#660]: https://github.com/amethyst/amethyst/pull/660
[#671]: https://github.com/amethyst/amethyst/pull/671
[#689]: https://github.com/amethyst/amethyst/pull/689
[#691]: https://github.com/amethyst/amethyst/pull/691
[#698]: https://github.com/amethyst/amethyst/pull/698
[#699]: https://github.com/amethyst/amethyst/pull/699
[#662]: https://github.com/amethyst/amethyst/pull/662
[#613]: https://github.com/amethyst/amethyst/pull/613
[#700]: https://github.com/amethyst/amethyst/pull/700

## [0.5.1] - 2017-08-30

* Fix syntax highlighting in documentation.

## [0.5.0] - 2017-08-29

### Added

* Add audio support ([#265])

### Changed

* Asset management rewrite (pull request [#244]).
* Use RON as config format ([#269])
* Overhaul input system ([#247]), ([#261]), and ([#274])
* Total overhaul of the game renderer ([#285])

[#244]: https://github.com/amethyst/amethyst/pull/244
[#247]: https://github.com/amethyst/amethyst/pull/247
[#261]: https://github.com/amethyst/amethyst/pull/261
[#265]: https://github.com/amethyst/amethyst/pull/265
[#269]: https://github.com/amethyst/amethyst/pull/269
[#274]: https://github.com/amethyst/amethyst/pull/274
[#285]: https://github.com/amethyst/amethyst/pull/285

## [0.4.3] - 2017-06-03

### Added

* Add mouse button events to `InputHandler` (pull request [#181]).
* Built-in application profiler using [`thread_profiler`][tp] (pull request
  [#212]).
* Screenshots for all in-repo examples (pull request [#213]).
* Pre-commit hook to automate local testing for commits (pull request [#228]).

### Changed

* Changes to `CONTRIBUTING.md` (pull requests [#206], [#226]).
* Update to `specs` 0.8.1 (pull request [#219]).

### Fixed

* Fix deferred rendering in renderable example (pull request [#211]).
* Fix AppVeyor curl command (pull request [#217]).
* Ignore IntelliJ IDEA project files (pull request [#218]).
* Fix `InputHandler` key press bug (pull request [#227]).
* Fix CRLF normalization on extensionless files (pull request [#207]).
* Update code to latest template (pull request [#215]).

[#181]: https://github.com/amethyst/amethyst/pull/181
[#206]: https://github.com/amethyst/amethyst/pull/206
[#207]: https://github.com/amethyst/amethyst/pull/207
[#211]: https://github.com/amethyst/amethyst/pull/211
[#212]: https://github.com/amethyst/amethyst/pull/212
[#213]: https://github.com/amethyst/amethyst/pull/213
[#215]: https://github.com/amethyst/amethyst/pull/215
[#217]: https://github.com/amethyst/amethyst/pull/217
[#218]: https://github.com/amethyst/amethyst/pull/218
[#219]: https://github.com/amethyst/amethyst/pull/219
[#226]: https://github.com/amethyst/amethyst/pull/226
[#228]: https://github.com/amethyst/amethyst/pull/228
[#227]: https://github.com/amethyst/amethyst/pull/227
[tp]: https://github.com/glennw/thread_profiler

## [0.4.2] - 2017-03-07

### Added

* Allow loading configuration files directly from strings.
* Add `#[derive(Default)]` for some types in ECS module.
* Add Ilya Bogdanov, Konstantin Zverev, and Scott Corbeil to `AUTHORS.md`.

### Changed

* Implement some clippy suggestions.
* Use `FnvHasher` instead of Rust's default SipHash implementation for better
  performance.

### Fixed

* Correct the quick example given in `README.md`.
* Replace constant paddle width with actual value in Pong example.
* Minor fix of line numbers in link in `CONTRIBUTING.md`.
* Add backticks around word in doc comment within `input.rs`.
* Match `Stopwatch` behavior to API documentation.
* Fix AppVeyor build failures due to `timing.rs` test failure.

## [0.4.1] - 2017-02-10

### Added

* Make `CONTRIBUTING.md` have teeth by enabling `#[deny(missing_docs)]`.
* Add lots of shiny new API documentation.
* Convert `amethyst` crate into a workspace.
* Add Travis and Appveyor badges to Cargo manifests.

### Changed

* Bump `amethyst` to version 0.4.1, `amethyst_renderer` to 0.4.1, and
  `amethyst_config` to 0.2.1.
* Temporarily disable `cargo fmt` checking in Travis due to panics.
* Update to `dds` 0.4.
* Update to `gfx` 0.14, fix breaking changes relating to shaders, PSO, and
  module layout changes.
* Update to `gfx_device_gl` 0.13.
* Update to `gfx_window_glutin` 0.14.
* Update to `glutin` 0.7.
* Improve quality of existing doc comments.
* Implement `Deref` and `DerefMut` into `glutin::Event` for `WindowEvent`.
* Re-export contents of `engine` to top-level and make module private.
* Shorten certain variable names to help combat rightward drift.
* Update `.travis.yml` and `appveyor.yml` to use `cargo test --all` instead of
  specifying explicit crates.
* Rename `06_assets` to `05_assets`.
* Make Git line endings consistent for source and config files throughout the
  repo.
* Process entire codebase through `cargo fmt`.
* Improve wording and formatting in `CONTRIBUTING.md` and in `README.md`.

### Removed

* Delete `rustfmt.toml` from `amethyst_renderer`.
* Delete outdated example from `amethyst_renderer`.
* Delete redundant `extern crate` directives outside of `lib.rs`.

## [0.4.0] - 2017-02-07

### Added

* Add transform system, transform components, light components, `specs`
  resources (camera, input handler, game time counter, screen dimensions, event
  handling).
* Make mesh primitives with [genmesh][gm].
* Add basic asset management.
  * Add support for Wavefront OBJ assets with [wavefront_obj][wo], and
    texture loading with [imagefmt][if].
  * Add support for DirectDraw surfaces (.dds files).
* Moar examples! Oh, and we have a [basic pong game][pg] too.
* Fix several `unused_variables` and `unused_mut` warnings.
* Add gitattributes to prevent line-ending conversion for binary files.
* Add lots of API documentation.

[gm]: https://github.com/gfx-rs/genmesh
[wo]: https://github.com/PistonDevelopers/wavefront_obj
[if]: https://github.com/lgvz/imagefmt
[pg]: examples/pong/

### Changed

* Relicense under the terms of both MIT/Apache-2.0.
* Revamp `amethyst_renderer`
  * Graphics backend chosen at compile time using features.
  * Add specular lighting, switching propagation -> attenuation.
* Update instructions for generating a new project using Cargo templates.
* Scale number of `specs` threads according to system core count.
* Improve Travis CI build speeds.
* Rewrite `Stopwatch` to be an enum.
* Update contribution guidelines and change log.
* Update book to reflect new API changes.
* Update dependency versions.

### Removed

* Remove `amethyst_ecs` crate in favor of using `specs` directly.
* Remove `amethyst_context` and refactor to greatly improve performance.
* Remove unused lights from included forward and deferred renderer pipelines.
* Remove dependency on `time` crate.

## [0.3.1] - 2016-09-07

### Fixed

* Fixed broken API reference link in `README.md`.
* amethyst.rs book: link to API reference broken (issue [#86]).
* Master branch no longer builds on beta/nightly Rust (issue [#94]).

[#86]: https://github.com/amethyst/amethyst/issues/86
[#94]: https://github.com/amethyst/amethyst/issues/94

## 0.3.0 - 2016-03-31

### Added

* Initial version of `amethyst_ecs` crate (issue [#37]).
* Add Gitter webhooks support to Travis (issue [#27]).

### Changed

* Update `amethyst_renderer` crate slightly (issue [#37]).
* Remove `publish.sh` script since website repo handles docs now (issue [#27]).
* Updated contribution guidelines on submitting code (issue [#37]).

### Fixed

* Update broken links for website, wiki, chat, and blog (issue [#27]).

[#27]: https://github.com/amethyst/amethyst/issues/27
[#37]: https://github.com/amethyst/amethyst/issues/37

## 0.2.1 (2016-01-27)
### Changed

* Add keywords to sub-crates.
* Remove reference to missing README file from `amethyst_engine`

## 0.2.0 (2016-01-27) [YANKED]
### Added

* Pass slice references to functions instead of `&Vec<T>`.
* Add state machine unit tests (issue [#9], pull request [#15])

### Changed

* Mention nightly Rust in "Hello World" tutorial (issue [#11], pull request
  [#12])
* Split amethyst` into separate sub-crates (issue [#13], pull request [#14])
* Update example to reflect API changes
* Depend on gfx-rs to reduce workload and foster cooperation, removed old
  renderer backend code

[#9]: https://github.com/amethyst/amethyst/issues/9
[#11]: https://github.com/amethyst/amethyst/issues/11
[#12]: https://github.com/amethyst/amethyst/issues/12
[#13]: https://github.com/amethyst/amethyst/issues/13
[#14]: https://github.com/amethyst/amethyst/issues/14
[#15]: https://github.com/amethyst/amethyst/issues/15

## 0.1.4 - 2016-01-10

### Added

* Stabilize state machine API (pull request [#6]).
  * Implement pushdown automaton state machine.
  * Implement state transitions.

### Changed

* Remove standardized `State` constructor (pull request [#6]).
* Update book and doc comments.

[#6]: https://github.com/amethyst/amethyst/issues/6

### Fixed

* Fix unreachable shutdown statement bug (issue [#5]).

[#5]: https://github.com/amethyst/amethyst/issues/5

## 0.1.3 - 2016-01-09

### Changed

* Clean up use statements.
* Renderer design progress (issue [#7]).
  * Split `ir.rs` and `frontend.rs` into separate files.
  * Frontend
    * Objects and Lights (enums) are now structs impl'ing `Renderable` trait.
    * `Frame` is a container of `Renderable` trait objects.
    * Start compiling library of common objects and light types.
  * Intermediate Representation
    * Move GPU state modeling out of Backend and into IR.
    * CommandBuffers are now directly sortable.
    * CommandQueue now takes in CommandBuffers directly
  * Backend
    * Consolidate traits into one short file.

[#7]: https://github.com/amethyst/amethyst/issues/7

## 0.1.1 - 2016-01-06

### Added

* Add `Frame::with_data` constructor to renderer.

### Changed

* Hide engine submodule, reexport desired contents as public.
* Updated hello_world.rs to new API.
* Significantly expanded Amethyst book and doc comments.

## 0.1.0 - 2016-01-03

* Initial release

[Unreleased]: https://github.com/amethyst/amethyst/compare/v0.11.0...HEAD
[0.11.0]: https://github.com/amethyst/amethyst/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/amethyst/amethyst/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/amethyst/amethyst/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/amethyst/amethyst/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/amethyst/amethyst/compare/v0.5.1...v0.7.0
[0.5.1]: https://github.com/amethyst/amethyst/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/amethyst/amethyst/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/amethyst/amethyst/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/amethyst/amethyst/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/amethyst/amethyst/compare/v0.4...v0.4.1
[0.4.0]: https://github.com/amethyst/amethyst/compare/v0.3.1...v0.4
[0.3.1]: https://github.com/amethyst/amethyst/compare/v0.3...v0.3.1
