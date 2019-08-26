# Examples

All examples can be run with the following command

```
cargo run --example name_of_an_example
```

---
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

Shows the basics of the state machine in Amethyst, and prints the following
```
Begin!
Hello from Amethyst!
End!
```

## Window

Opens a window and creates a render context. Additionally shows basic raw input handling.

![window example result](assets/img/window.png)

### Custom Game Data

Demonstrates how to use custom `GameData`, with three different states: `Loading`, `Main`, `Paused`.

![game_data_example_result](custom_game_data/screenshot.png)

### Sphere

Render a basic 3D scene, with a camera, lights and a 3D object, a sphere in this scenario.
This example use a single vertex buffer with all attributes interleaved.

![sphere example result](assets/img/sphere.png)

### Separate sphere

Render a basic 3D scene, with a camera, lights and a 3D object, a sphere in this scenario.
This example use vertex buffers per attribute.

![sphere example result](assets/img/sphere.png)

### Multisample sphere

Render a basic 3D scene, with a camera, lights and a 3D object, a sphere in this scenario.
This example use vertex buffers per attribute.
Only difference here is that multisampling is enabled in the options.

![sphere example result](assets/img/sphere.png)

### Renderable

Load graphics objects from disc using the asset loader.
Also contains a custom system that move the camera and the scene.

![renderable example result](assets/img/renderable.png)

### Asset custom

Load a custom asset using a custom format.

### Asset loading

Create a custom format, and use the asset loader to load assets using the format.

![asset loading example result](assets/img/asset_loading.png)

### Material

Render a sphere using a physically based material.

![material example result](assets/img/material.png)

### Animation

Animate a sphere using a custom built animation sampler sequence. Keybindings:

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

### Gltf

Load a GLTF asset, attach it to an entity, and animate the asset. Press `Space` to start/pause the animation.

![gltf example result](assets/img/gltf.png)

### UI

Render a basic UI.

![ui example result](assets/img/ui.png)

### Pong

`Amethyst` based Pong clone. In addition to using most of the features used by the other examples it also demonstrates:

* Input handling using `InputHandler`
* Background music and sound effects
* A more interesting UI example
* A bigger project with more than a single source file.

![pong example result](assets/img/pong.png)

### Appendix A

From the book, it is a minor update to the Pong example that uses `Config` files instead of hardcoded constants.

![example screenshot](appendix_a/screenshot.png)

### Fly Camera

This example shows how to use the Fly Camera, how to release and capture back user mouse input, and show and hide mouse cursor.

### Arc ball Camera

This example shows how to use the Arc Ball Camera.

### Sprites Ordered

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

### Prefab

Shows how to load data using the `Prefab` system.

### Prefab Adapter

Shows how to create a `PrefabData` using the adapter pattern.

### Prefab Basic

Shows how to create a trivial `PrefabData` and instantiate an entity using the `Prefab` system.

### Prefab Multi

Shows how to create a `PrefabData` and instantiate an entity with multiple components using the `Prefab` system.

### Prefab Custom

Shows how to create a `PrefabData` and instantiate multiple entities with different components using the `Prefab` system.
