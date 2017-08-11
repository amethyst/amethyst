# Change Log
All notable changes to this project will be documented in this file. 

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## [Unreleased]
### Changed
* Asset management rewrite (pull request [#244]).
* Use RON as config format ([#269])
* Overhaul input system ([#247]), ([#261]), and ([#274])

[#244]: https://github.com/amethyst/amethyst/pull/244
[#247]: https://github.com/amethyst/amethyst/pull/247
[#261]: https://github.com/amethyst/amethyst/pull/261
[#269]: https://github.com/amethyst/amethyst/pull/269
[#274]: https://github.com/amethyst/amethyst/pull/274



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
[pg]: examples/04_pong/

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

[Unreleased]: https://github.com/amethyst/amethyst/compare/v0.4.2...HEAD
[0.4.3]: https://github.com/amethyst/amethyst/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/amethyst/amethyst/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/amethyst/amethyst/compare/v0.4...v0.4.1
[0.4.0]: https://github.com/amethyst/amethyst/compare/v0.3.1...v0.4
[0.3.1]: https://github.com/amethyst/amethyst/compare/v0.3...v0.3.1
