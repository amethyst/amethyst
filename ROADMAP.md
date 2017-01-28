## Vision

> Amethyst aims to be a modular, parallel and data-driven game engine

**Modularity** The game engine should be easily extendable to fit the needs of the game developers for their game idea. Therefore we expose an entity-component system, which allows to define new components and processors without needing to change the game engine code. This should emphasize the 'game engine as library' aspect.

**Parallelism** Current and future CPUs are multi-core processors, which can run multiple tasks in parallel. Rust provides us useful language features to implement parallelism, which should be used to provide an easy and safe API for the engine.

**Data-Driven** In order to achieve high performance, we will also have to take care of data-orientation next to parallelism. This is closely coupled with the ECS system: processors will perform certain operations over large data sets.

## Goals
These goals are mainly based on priority and will run through several iterations over the whole development process. So, short-term goals won't be finished when starting with mid-term content but still be worked on.
### Short term
These are the core functionalities for the game engine required to build a game. Mainly focuses on using the engine as library to build upon.
* Entity-Component System
* Rendering
* Input Handling
* Audio
* Serialization/Reflection
* Streaming/Asset Handling
* Documentation/Tutorial

### Mid-term

* Physics
* Tooling
* Animation

### Long-term

* Scripting
* Graphical UI
* Networking
* AI

### Develop A Game
A game engine is built in order to build games! The best tutorial for potential users are working examples, and fancier examples are even better.
Under this assumption we should also try to spend some time on translating our implementations into a working game, showcasing the technical aspects of our engine.

## Systems/Aspects
Discussing the scopes and features of the different subsystems in more detail.

### Entity-Component System
Our current system is based on `specs` and should probably stick to it in near future (in case some magically new library pops up..).
As the ECS is the core part of the architecture, this needs be carefully designed and should be clear under our 3 main vision aspects:

**Modularity** We expose the API from `specs` to the developers, in order to allow them to implement new components, processors and resources (larger singleton objects). The engine will have built-in processors for common operations like `Transform` to ensure spatial hierarchy.

**Parallelism** Processor run in parallel with respect to task- and data-dependencies with low overhead.

**Data-Driven** TODO

**Next steps**
 * Evaluating different approaches for defining task-dependencies between the processors, including support to run a processor multiple times
 * Research ways to handle single-threaded processors (e.g nphysics) if possible

### Rendering
The trend in industry is to achieve realistic rendering, therefore most engines implement physically based rendering.
Using `gfx-rs` for backend abstraction allows us to provide compatibility with multiple platforms without caring that much about the different graphic APIs

**Modularity** TODO: easily integrate own rendering approaches
**Parallelism** TODO: parallel command list creation
**Data-Driven** TODO: efficient rendering -> minimizing draw calls and overhead

**Next steps**
 * Transfer to physically based rendering (pretty large tasks..)
 * Evaluate current system in terms of extendability for developers to implement own rendering approaches
 * Feedback practical experience into improvements for `gfx-rs`

### Input Handling
**Modularity** The API should be flexible enough to support multiple different input devices including gamepads and maybe VR devices.
**Data-Driven** TODO: raw input

**Next steps**
 * Raw input support (winit?)
 * Gamepad support (winit?)
 * Input Mapping

### Audio
TODO: Unfortunately I'm not that deep into the audio components of game engines (state of art?)
I imagine some kind of OpenAL like API, supporting spatial sound source objects, audio streaming, attenuation, etc.
Provide low-level access?

** Next Steps**

### Serialization/Reflection
Type reflection offers additional information about the type, which is available at compile- or run-time. In comparison to C++, to can be achieved easier with Rusts macro system.
In our engine we want to couple the ECS with serialization/reflection to easily load/save game scenes from various formats. Additionally, it helps us in to build safe network protocols, writing script and GUI bindings.

** Next Steps**
 * Integrate serde based reflection
 * Serialize/Deserialize ecs worlds

### Streaming/Asset Handling
TODO

**Parallelism** TODO: Load assets in parallel
**Data-Driven** TODO: low loading overhead

** Next Steps**
 * Improve current approach regarding robustness (error handling) and flexibility with scene formats
 * Implement loaders for common file formats and archives

### Documentation/Tutorial
TODO

** Next Steps**

### Physics
TODO: I would consider physics pretty difficult part to do *right*, therefore not part of the short term goals.
Maybe rely on external libraries first.
Should still provide ways for collision detection, allowing to do collision checks, raycasts, etc. (-> ncollide?).

**Parallelism** TODO: internal physic parallelism, possible GPU implementation (hard), difficult with external libraries
**Data-Driven** TODO: integration issue: physics world <-> ECS world, having 2 different representations? if so how to keep them in sync and integrate with other conflicting processors (e.g spatial hierarchy)

** Next Steps**
 * Integrate an existing library or write from scratch (requires further planning then)
 * Provide collision testing API (triggers, raycasts, etc.)

### Tooling
We want to focus on developing a game engine, even though a rich toolset is required to simplify different task for users. If possible, we can integrate existing tools like profilers and 3D editors.
'In-house' tools would be implemented on demand, e.g. engine specific data formats, own editor (Qt based?).

** Next Steps**
 * Integrate better logging (high priority -> short-term)

### Scripting
Possible scripting languages: mruby, lua, moonscript, dyon, ..

Supporting a scripting language would support rapid-prototyping and faster iteration cycles. Also would be more attractive for developers who are not so much familiar with Rust.
Scripting could be use for several subsystems like animation, particle system, AI, etc.

** Next Steps**
 * Evaluate the different scripting languages with respect to interoperability, performance, usability, ..
 
### Graphical UI
** Next Steps**
 * Compatibility of GUIs with entity component systems (UI elements as entities <-> only root UI elements)
 * Integrate an existing UI library (e.g conrod) or develop a new library from scratch (requires further planning then)
 
Future:
A common trend for large engines is embedding webbrowser frameworks like CEF for UI. Allowing to re-use existing JS/HTML/CSS frameworks to create complex UIs in a short time.
Unfortunately CEF is pretty large, therefore a Servo based approach would be desirable for performance and easier integration.

### Networking
TODO: same as physics, hard to do correctly, interesting would be something like `libyojimbo` but well..

** Next Steps**

### AI
TODO: also no real experience with this..

** Next Steps**
