# Change Log

All notable per-release changes will be documented in this file. This project
adheres to [Semantic Versioning][sv].

[sv]: http://semver.org/

## 0.3.1 (2016-09-07)

### Fixed
* Fixed broken API reference link in `README.md`
* amethyst.rs book: link to API reference broken (issue [#86])
* Master branch no longer builds on beta/nightly Rust (issue [#94])

[#86]: https://github.com/amethyst/amethyst/issues/86
[#94]: https://github.com/amethyst/amethyst/issues/94

## 0.3.0 (2016-03-31)

### Added
* Initial version of `amethyst_ecs` crate (issue [#37])
* Add Gitter webhooks support to Travis (issue [#27])

### Changed
* Update `amethyst_renderer` crate slightly (issue [#37])
* Remove `publish.sh` script since website repo handles docs now (issue [#27])
* Updated contribution guidelines on submitting code (issue [#37])

### Fixed
* Update broken links for website, wiki, chat, and blog (issue [#27])

[#27]: https://github.com/amethyst/amethyst/issues/27
[#37]: https://github.com/amethyst/amethyst/issues/37

## 0.2.1 (2016-01-27)

### Changed
* Added keywords to sub-crates
* Removed reference to missing README file from `amethyst_engine`

## 0.2.0 (2016-01-27) [YANKED]

### Added
* Pass slice references to functions instead of `&Vec<T>`
* State machine gained some unit tests (issue [#9], pull request [#15])

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

## 0.1.4 (2016-01-10)

### Added
* Stabilize state machine API (pull request [#6])
  * Implement pushdown automaton state machine
  * Implement state transitions

### Changed
* Remove standardized `State` constructor (pull request [#6])
* Update book and doc comments

[#6]: https://github.com/amethyst/amethyst/issues/6

### Fixed
* Fix unreachable shutdown statement bug (issue [#5])

[#5]: https://github.com/amethyst/amethyst/issues/5

## 0.1.3 (2016-01-09)

### Changed
* Clean up use statements
* Renderer design progress (issue [#7])
  * Split ir.rs and frontend.rs into separate files
  * Frontend
    * Objects and Lights (enums) are now structs impl'ing `Renderable` trait
    * `Frame` is a container of `Renderable` trait objects
    * Start compiling library of common objects and light types
  * Intermediate Representation
    * Move GPU state modeling out of Backend and into IR
    * CommandBuffers are now directly sortable
    * CommandQueue now takes in CommandBuffers directly
  * Backend
    * Consolidate traits into one short file

[#7]: https://github.com/amethyst/amethyst/issues/7

## 0.1.1 (2016-01-06)

### Added
* Add `Frame::with_data` constructor to renderer

### Changed
* Hide engine submodule, reexport desired contents as public
* Updated hello_world.rs to new API
* Significantly expanded Amethyst book and doc comments

## 0.1.0 (2016-01-03)

* Initial release
