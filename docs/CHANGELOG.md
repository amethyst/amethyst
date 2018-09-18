# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## Unreleased
### Added
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

### Changed
* Sprites contain their dimensions and offsets to render them with the right size and desired position. ([#829], [#830])
* Texture coordinates for sprites are 1.0 at the top of the texture and 0.0 at the bottom. ([#829], [#830])
* Made get_camera public. ([#878)]
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

### Removed
* `LMenu` and `RMenu` key codes, following the `winit` update. ([#906])

### Fixed
* Material ids in GLTF loader caused multiple GLTF files to get incorrect materials applied. ([#915])
* Fix render gamma for most textures. ([#868])
* Joint entities can only be part of a single skin: Materials are not swapped anymore. ([#933])
* Fixed regression in sprite positioning after batching. ([#929])

[#829]: https://github.com/amethyst/amethyst/issues/829
[#830]: https://github.com/amethyst/amethyst/pull/830
[#879]: https://github.com/amethyst/amethyst/pull/879
[#878]: https://github.com/amethyst/amethyst/pull/878
[#887]: https://github.com/amethyst/amethyst/pull/887
[#892]: https://github.com/amethyst/amethyst/pull/892
[#877]: https://github.com/amethyst/amethyst/pull/877
[#896]: https://github.com/amethyst/amethyst/pull/896
[#831]: https://github.com/amethyst/amethyst/pull/831
[#902]: https://github.com/amethyst/amethyst/pull/902
[#905]: https://github.com/amethyst/amethyst/pull/905
[#920]: https://github.com/amethyst/amethyst/pull/920
[#903]: https://github.com/amethyst/amethyst/issues/903
[#904]: https://github.com/amethyst/amethyst/pull/904
[#915]: https://github.com/amethyst/amethyst/pull/915
[#868]: https://github.com/amethyst/amethyst/pull/868
[#933]: https://github.com/amethyst/amethyst/pull/933
[#929]: https://github.com/amethyst/amethyst/pull/929
[#934]: https://github.com/amethyst/amethyst/pull/934
[#940]: https://github.com/amethyst/amethyst/pull/940
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

[Unreleased]: https://github.com/amethyst/amethyst/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/amethyst/amethyst/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/amethyst/amethyst/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/amethyst/amethyst/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/amethyst/amethyst/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/amethyst/amethyst/compare/v0.4...v0.4.1
[0.4.0]: https://github.com/amethyst/amethyst/compare/v0.3.1...v0.4
[0.3.1]: https://github.com/amethyst/amethyst/compare/v0.3...v0.3.1
