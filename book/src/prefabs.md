# Prefabs

Premade fabrications, or prefabs, are templates that specify components to attach to an entity. For example, imagine a monster entity has the following components:

- Position
- Velocity
- Health points
- Attack damage

It is certainly possible to define the values for each monster in code. However, if the components are initialized in code, then the executable needs to be recompiled whenever the component values are changed. Waiting a number of minutes to recompile every time a small change is made is both inefficient and frustrating.

In data-oriented design, instantiating monsters is logic and is part of the executable, but the values to use for the components is data. The executable can instantiate any monster using the data for the monsters read from a *prefab* file like the following:

```rust ,ignore
// This is simply an example of what a prefab can look like.
// Other game engines may store prefabs in binary formats which require
// an editor to read and update.
Prefab( 
    id: "00000000-0000-0000-0000-000000000000",  // this is the unique id of this prefab in your game
    objects: [ // each entity is represented by an object in this array
        Entity(( // each Entity is described by an UUID and list of components
             id: "00000000-0000-0000-0000-000000000000",  // this uniquely identifies this entity
             components: [  // you can have any number of components for an entity
                 (
                     type: "00000000-0000-0000-0000-000000000000",  // this is the UUID of the particular component type
                     data: ( // these are the fields of the component struct
                        position: (0.0, 0.0, 0.0),
                        velocity: (0.0, 0.0, 0.0),
                        health: 100,
                        attack: 10,
                     ),
                 ),
             ]
        )),
        Entity(( // you may have multiple entities in a prefab
             id: "00000000-0000-0000-0000-000000000000",
             components: [
                 (
                     type: "00000000-0000-0000-0000-000000000000",
                     data: (
                         position: [200.0, 200.0]
                     ),
                 ),
             ]
        )),
    ]
)
```

The prefab is distributed alongside the executable as part of the game or baked in to a binary format for release.

## Uses

Prefabs have the following properties:

- All entity instances created based on that prefab will receive changes made on the prefab.
- Prefabs may nest other prefabs, allowing larger prefabs to be composed of other smaller prefabs.

These make prefabs ideal to use to define scenes or levels:

- City prefab composed of terrain, buildings, and foliage prefabs.
- Maze prefab composed of walls, a player, and monster prefabs.
