# Renderer Design

Central to a visually impressive game is a powerful renderer. The document below
describes the design decisions behind the rendering engine developed for
Amethyst.

## Goals

* Develop a portable real-time 3D renderer with support for multiple backends.
  * Priority: Vulkan 1.0+, OpenGL 4.3+
  * Others are nice to have.
* Design a simple and high-level submission format. Clients should never
  directly alter low-level GPU state in code.
  * Frames, objects, and lights.
* Embrace data-driven design from the start.
  * No rendering approach built in, users load it in as a data structure.
  * Objects, lights, and render paths should be easy to serialize and
    de-serialize.
* Internal abstractions should take inspiration from modern graphics API design
  patterns. Next-gen ready, backwards compatible with legacy stateful APIs like
  OpenGL.
  * Static state objects created once on startup and cached (i.e. pipelines).
  * Dynamic state objects updated per-frame (i.e. blend, depth-stencil,
    rasterizer, viewport).
  * [Stateless draw calls][st].
  * Multithreading used judiciously throughout to maximize performance.
* Make this renderer usable outside of the Amethyst engine itself.

[st]: http://blog.molecular-matters.com/2014/11/06/stateless-layered-multi-threaded-rendering-part-1/

## Non-Goals

* Provide a game loop or event/message pump.
  * Argument: Should be responsibility of the game engine or client application.
    Renderer code should be reusable and easy to integrate into a variety of
    third-party application frameworks.
* Implement a scene graph or "game world" data structure.
  * Argument: Separation of concerns. The renderer should only know how to draw
    a set of objects on screen, and it should not force a particular scene model
    on its users. Whether one prefers use a traditional class hierarchy over an
    entity-component-system model, or vice versa, should not matter.
* Implement a default spatial data structure, e.g. an octree or BSP.
  * Argument: Scene culling methods all have different performance requirements.
    The user should decide what spatial data structure is most suitable for
    their needs.
* Implement or provide a default serialization file format parser, e.g. XML,
  JSON, or YAML.
  * Argument: The user can choose to represent their render path in a manner
    most suitable to their application, whether that be in an external file or
    hard-coded in Rust.

## High-Level Design

The final design loosely resembles the approach used by [Bitsquid][bs] (now
Autodesk Stingray), the [Oxide Engine][oe], and [id Tech 4][it]. There are three
main parts:

[bs]: http://bitsquid.blogspot.com/2009/10/parallel-rendering.html
[oe]: http://www.gdcvault.com/play/1020706/Nitrous-Mantle-Combining-Efficient-Engine
[it]: http://fabiensanglard.net/doom3/renderer.php

1. Frontend
   * Loads a `RenderPath` structure upon initialization. Uses it at runtime to
     convert a given `Frame` into an intermediate representation (IR).
   * Sorts the IR to minimize redundant state changes.
2. Intermediate Representation
   * A set of data structures that represent the high-level actions that the GPU
     must take in order to render a single `Frame`.
   * Essentially stateless draw calls. Similar to command buffers in Vulkan, but
     API-agnostic.
3. Backend
   * Translates the IR received from the frontend into low-level API calls.
   * One backend for each target API.
   * Backends are selected by users at compile time. For the sake of runtime
     overhead and stability, dynamic switching of backends is not supported.

![Overall](./internals/overall.png)

## Example Usage

```rust
//! This is a dummy program making direct use of the renderer.

extern crate amethyst;
extern crate amethyst_opengl;

use amethyst::renderer::*;
use amethyst_opengl::BackendGl;

fn main() {
    // Describe our render path, e.g. forward, deferred, depth pre-pass, etc.
    let mut example = RenderPath { ... };

    // Initialize the renderer with our choice of backend.
    let mut frontend = Frontend::new(BackendGl);
    frontend.use_path(example);

    loop {
        // Collect all data needed for rendering.
        let mut frame = Frame { ... };

        // Draw!
        frontend.draw(frame);
    }
}
```

See the next three sections to learn how the frontend, IR, and backend all work
in more detail.
