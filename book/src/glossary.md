# Glossary

## Data-driven design

Describes a program that has its logic defined largely in data rather than in
compiled code. Ideally, this would permit the user to edit their code and
resources using offline tools and have the program hot-reload the changes at
run-time for instant feedback without the need for recompilation. The bare
minimum qualification for a data-driven program is the ability to read external
content (text files, scripts, byte streams) and mutate its behavior accordingly.

## Data-oriented programming

Not to be confused with data-driven design, data-oriented programming is a
programming paradigm, like object-oriented programming (OOP) or procedural
programming. Where OOP focuses on modeling a problem in terms of interacting
objects, and procedural programming tries to model a problem in terms of
sequential or recursive steps or procedures, data-oriented programming shifts
the focus towards the data being operated on: the data type, its memory layout,
how it will be processed. Software written in a data-oriented manner tends
toward high-throughput pipelining, modularity, separation of concerns, and
massive parallelism. If architected correctly, data-oriented software can be
very cache-friendly and easy to scale on systems with multiple cores.

> Note: Data-oriented programming does not necessarily imply that a program is
> data-driven. Data-driven behavior can be implemented with any programming
> approach you like.

## Entity-component-system (ECS) model

Describes a game programming design pattern invented as a reaction to the
deep-rooted problems with using *inheritance* (is-a relationship) to represent
game objects, including the [deadly diamond of death][dd] and [god objects][go].
The inheritance-based approach was especially common in the game industry during
the 1990's and early 2000's.

This alternative model makes use of *composition* (has-a relationship) instead
of inheritance to represent objects in the game world, flattening the hierarchy
and eliminating the problems above, while increasing flexibility. The holistic
ECS approach is broken into three key pieces:

1. *Entity*: Represents a single object in the game world. Has no functionality
   on its own. The world owns a collection of entities (either in a flat list or
   a hierarchy). Each entity has a unique identifier or name, for the sake of
   ease of use.
1. *Component*: A plain-old-data structure that describes a certain trait an
   entity can have. Can be "attached" to entities to grant them certain
   abilities, e.g. a `Light` component contains parameters to make an entity
   glow, or a `Collidable` component can grant an entity collision detection
   properties. These components *do not* have any logic. They contain only data.
1. *System*: This is where the magic happens! Systems are centralized game engine
   subsystems that perform a specific function, such as rendering, physics, audio,
   etc. Every frame, they process each entity in the game world looking for
   components that are relevant to them, reading their contents, and performing
   actions. For example, a `Rendering` system could search for all entities that have
   `Light`, `Mesh`, or `Emitter` components and draw them to the screen.

This approach could potentially be stretched to fit the model-view-controller
(MVC) paradigm popular in GUI and Web development circles: entities and
components together represent the model, and systems represent either views
(`Rendering`, `Audio`) or controllers (`Input`, `AI`, `Physics`), depending on
their purpose.

Another great advantage of the ECS model is the ability to rapidly prototype
a game simply by describing objects' characteristics in terms of creating
entities and attaching components to them, with very little game code involved.
And all of this data can be easily serialized or de-serialized into a
human-friendly plain text format like [RON] (Json derivative).

For more detailed explanations of entity-component-system designs, please
[see this great post on Reddit][p1] and [this Stack Overflow answer][p2].

[dd]: https://en.wikipedia.org/wiki/Multiple_inheritance#The_diamond_problem
[go]: https://en.wikipedia.org/wiki/God_object
[p1]: https://www.reddit.com/r/rust/comments/43p2fq/this_week_in_amethyst_3/czkc4hj
[p2]: http://gamedev.stackexchange.com/questions/31473/what-is-the-role-of-systems-in-a-component-based-entity-architecture/31491#31491
[ron]: https://github.com/ron-rs/ron
