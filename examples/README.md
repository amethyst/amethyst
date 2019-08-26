# Examples

All examples can be run with the following command

```
cargo run --example name_of_an_example
```

---

## Table of Contents

1. Basic
   1. [Hello World](#hello-world)
   2. [Window](#window)
   3. [Custom Game Data](custom-game-data)
2. Rendering
   1. [Sphere](#sphere)
   2. [Separate Sphere](#separate-sphere)
   3. [Multisample Sphere](#multisample-sphere)
   4. [Renderable](#renderable)
3. Assets
   1. [Asset Custom](#asset-custom)
   2. [Asset Loading](#asset-loading)
   3. [Material](#material)
   4. [Animation](#animation)
   5. [GLTF](#gltf)
   6. Prefabs
      1. [Prefab Adapter](#prefab-adapter)
      2. [Prefab Basic](#prefab-basic)
      3. [Prefab Multi](#prefab-multi)
      4. [Prefab Custom](#prefab-custom)
4. UI
   1. [UI](#ui)
5. Miscellaneous
   1. [Fly Camera](#fly-camera)
   2. [Arc ball Camera](#arc-ball-camera)
   3. [Sprites Ordered](#sprites-ordered)
6. Games
   1. [Pong](#pong)
   2. [Appendix A](#appendix-a)

## Hello World

Basic state machine in Amethyst. Prints the following:

```
Begin!
Hello from Amethyst!
End!
```

---

## Window

Opens a window and creates a render context. Additionally, shows basic raw input handling.

![window example screenshot](assets/img/window.png)

---

## Custom Game Data

Uses `GameData`, with three different states: `Loading`, `Main`, `Paused`.

![game data example screenshot](assets/img/custom-game-data.png)

---

## Sphere

Renders a basic sphere.

![sphere example screenshot](assets/img/sphere.png)

---

## Renderable

Loads graphics objects from disk using the asset loader. Contains a custom system that moves the camera and scene.

![renderable example screenshot](assets/img/renderable.png)

---

## Asset custom

Loads a custom asset using a custom format.

---

## Asset loading

Creates a custom format and loads it using the asset loader.

![asset loading example screenshot](assets/img/asset-loading.png)

---

## Material

Renders a sphere using a physically based material.

![material example screenshot](assets/img/material.png)

---

## Animation

Animates a sphere using a custom-built animation sampler sequence. Keybindings:

* `Space` - start/pause/unpause the currentanimation(default is translational animation)
* `D` - demonstrate deferred start, translate will run first, then rotate when translate ends, and last scale animation
        will start after rotation has run for 0.66s.
* `T` - set translate to current animation
* `R` - set rotate to current animation
* `S` - set scale to current animation
* `H` - run animation at half speed
* `F` - run animation at full speed
* `V` - run animation at no speed, use stepping keys for controlling the animation
* `Right` - step to the next animation keyframe
* `Left` - step to the previous animation keyframe

![animation example screenshot](assets/img/animation.png)

---

## GLTF

Loads a GLTF asset, attaches it to an entity, and animates the asset. Press `Space` to start/pause the animation.

![gltf example screenshot](assets/img/gltf.png)

---

## UI

Renders a basic UI.

![ui example screenshot](assets/img/ui.png)

---

## Pong

`Amethyst` based Pong clone. In addition to using most of the features used by the other examples, it also demonstrates:

* Input handling using `InputHandler`
* Background music and sound effects
* A more interesting UI example
* A larger, multi-file project

![pong example screenshot](assets/img/pong.png)

---

## Fly Camera

Shows the Fly Camera. Captures and releases mouse input.

![Fly Camera example screenshot](assets/img/fly-camera.png)

---

## Arc ball Camera

Shows the Arc Ball Camera.

![Fly Camera example screenshot](assets/img/arc-ball-camera.png)

---

## Sprites Ordered

Draws sprites ordered by Z coordinate. Entities with larger Z coordinates will have their sprites drawn in front of entities with smaller Z coordinates.

This example also demonstrates the use of the `Transparent` component, the depth buffer, and
camera depth values.

Keybindings:

* `T` - Toggle whether the `Transparent` component is attached to entities.
* `R` - Reverse the Z coordinates of the entities.
* `Up` - Increase the Z coordinate of the camera.
* `Down` - Decrease the Z coordinate of the camera.
* `Right` - Increase the depth (Z distance) that the camera can see.
* `Left` - Decrease the depth (Z distance) that the camera can see.

![Fly Camera example screenshot](assets/img/sprites-ordered.png)

---

## Prefab

Loads data using the `Prefab` system.

---

## Prefab Adapter

Creates a `PrefabData` using the adapter pattern.

---

## Prefab Basic

Creates a trivial `PrefabData` and instantiates an entity using the `Prefab` system.

---

## Prefab Multi

Creates a `PrefabData` and instantiates an entity with multiple components using the `Prefab` system.

---

## Prefab Custom

Create a `PrefabData` and instantiates multiple entities with different components using the `Prefab` system.
