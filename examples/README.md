# Examples

All examples can be run with the following command, where `{{name}}` is the name of the example, and `{{backend}}` is your graphics backend (usually `metal` on MacOS and `vulkan` on everything else.) Note that some examples require the additional `gltf` feature.

```
cargo run --example {{name}} --features "{{backend}}"
```

---

## Table of Contents

1. Basic
   1. [Hello World](hello_world)
   2. [Window](window)
   3. [Custom Game Data](custom_game_data)
   4. [Events](events)
   5. [State Dispatcher](state_dispatcher)
2. Rendering
   1. [Sphere](sphere)
   2. [Spotlights](spotlights)
   3. [Sprites Ordered](sprites_ordered)
   4. [Renderable](renderable)
   5. [rendy](rendy)
   5. [Custom Render Pass](custom_render_pass)
3. Assets
   1. [Asset Custom](asset_custom)
   2. [Asset Loading](asset_loading)
   3. [Material](material)
   4. [Animation](animation)
   5. [GLTF](gltf)
   6. Prefabs
      1. [Prefab Adapter](prefab_adapter)
      2. [Prefab Basic](prefab_basic)
      3. [Prefab Multi](prefab_multi)
      4. [Prefab Custom](prefab_custom)
4. UI
   1. [UI](ui)
   2. [Custom UI](custom_ui)
   3. [States Example](states_ui)
5.  Debugging
    1.  [Debug Lines](debug_lines)
    2.  [Debug Lines Ortho](debug_lines_ortho)
6.  Networking
    1.  [Net Client](net_client)
    2.  [Net Server](net_server)
7. Miscellaneous
   1. [Fly Camera](fly_camera)
   2. [Arc ball Camera](arc_ball_camera)
   3. [Auto FOV](auto_fov)
   4. [Sprite Camera Follow](sprite_camera_follow)
   5. [Locale](locale)
   6. [Tiles](tiles)
   7. [Optional graphics](optional_graphics)
8. Games
   1. [Pong 1](pong_tutorial_01)
   2. [Pong 2](pong_tutorial_02)
   3. [Pong 3](pong_tutorial_03)
   4. [Pong 4](pong_tutorial_04)
   5. [Pong 5](pong_tutorial_05)
   6. [Pong 6](pong_tutorial_06)
