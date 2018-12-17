# Prefabs

Premade fabrications, or prefabs, are templates that specify components to attach to an entity. For example, imagine a monster entity has the following components:

* Position
* Velocity
* Texture
* Health points
* Attack damage

It is certainly possible to define the values for each monster in code. However, if the components are initialized in code, then the executable needs to be recompiled whenever the component values are changed. Waiting a number of minutes to every time a small change is made is both inefficient and frustrating.

In data-oriented design, instantiating monsters is logic and is part of the executable, but the values to use for the components is data. The executable can instantiate any monster using the data for the monsters read from a *prefab* file like the following:

```rust,ignore
// monster_weak.ron
//
// This is simply an example of what a prefab can look like.
// Other game engines may store prefabs in binary formats which require
// an editor to read and update.

#![enable(implicit_some)]
Prefab (
    entities: [
        (
            data: (
                position: (0.0, 0.0, 0.0),
                velocity: (0.0, 0.0, 0.0),
                texture: Asset(File("textures/monster.png", PngFormat, ())),
                health: 100,
                attack: 10,
            ),
        ),
    ],
)
```

The prefab is distributed alongside the executable as part of the game.

## Uses

Prefabs have the following properties:

* All entity instances created based on that prefab will receive changes made on the prefab.
* Prefabs may nest other prefabs, allowing larger prefabs to be composed of other smaller prefabs.

These make prefabs ideal to use to define scenes or levels:

* City prefab composed of terrain, buildings, and foliage prefabs.
* Maze prefab composed of walls, a player, and monster prefabs.
