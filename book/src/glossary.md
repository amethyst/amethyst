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
