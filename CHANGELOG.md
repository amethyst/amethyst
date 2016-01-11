# Changelog

## 0.1.4 ()

* Core
  * Stabilize state management
    * Implement pushdown automaton state machine
    * State transitioning
    * Remove standardized State constructor
    * Update book and doc comments

## 0.1.3 (0ecc7d8c509f00915214386c6c74b6377b7d9fc3)

* Core
  * Clean up use statements
* Renderer
  * Split ir.rs and frontend.rs into separate files
  * Frontend
    * Objects and Lights (enums) are now structs impl'ing Renderable trait
    * Frame is a container of Renderable trait objects
    * Start compiling library of common objects and light types
  * Intermediate Representation
    * Move GPU state modeling out of Backend and into IR
    * CommandBuffers are now directly sortable
    * CommandQueue now takes in CommandBuffers directly
  * Backend
    * Consolidate traits into one short file

## 0.1.1 (b7d804acd3a51db096a1bebe56eb79b6dcc23351)

* Core
  * Hide engine submodule, reexport desired contents as public
* Docs
  * Significantly expanded Amethyst book and doc comments
  * Updated hello_world.rs to new API
* Renderer
  * Add Frame::with_data constructor

## 0.1.0

* Initial release
