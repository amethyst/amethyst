# The Frontend

Every renderer, real-time or not, needs a good frontend. A renderer can be fast,
flexible, or feature-rich, but if its interface isn't straightforward to new
users, few will bother to use it. This frontend tries to abstract away most of
the complexities plaguing real-time rendering and make it dead simple to
integrate the renderer into your existing application or game engine.

## Design

As hinted in the previous section, the renderer frontend is responsible for two
tasks:

1. Parsing a `RenderPath` structure on initialization and using it per-frame to
   convert each object and light into a single equivalent IR `CommandBuffer`.
2. Merging and sorting these command buffers to eliminate redundant state
   changes, and then shipping them off to the backend.

![Frontend](./internals/renderer/frontend.png)

## Usage

Drawing stuff is simple enough. The `draw()` method takes in a structure called
a `Frame`, which is a collection of objects and lights, and draws them.
