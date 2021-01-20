# Specs Migration

- Specs migration

  Quick fix:

  - Add `use amethyst::ecs::WorldExt` to imports.
  - Replace `world.add_resource` with `world.insert`.
  - `use amethyst::ecs::WorldExt;` for `world.read_resource`.
  - Regex replace `\bResources\b` with `World`. Check for false replacements.
  - Replace `world.res` with `world`.
  - Regex replace `\bres\b` with `world`.

  `shred-derive` is re-exported by `amethyst`. Migration steps:

  - Remove `shred-derive` from `Cargo.toml`.
  - Remove `use amethyst::ecs::SystemData` from imports (if present).
  - Add `use amethyst::shred::{ResourceId, SystemData}` to imports.

- `PrefabLoaderSystem` is initialized by `PrefabLoaderSystemDesc`.

  **Quick fix:**

  - Find: `PrefabLoaderSystem::<([A-Za-z]+)>::default\(\)`,
  - Replace: `PrefabLoaderSystemDesc::<\1>::default()`
  - Don't forget to replace `with` with `with_system_desc` when adding to GameData.

- `GltfSceneLoaderSystem` is initialized by `GltfSceneLoaderSystemDesc`.

  **Quick fix:**

  - Find: `GltfSceneLoaderSystem::<([A-Za-z]+)>::default\(\)`,
  - Replace: `GltfSceneLoaderSystemDesc::<\1>::default()`
  - Don't forget to replace `with` with `with_system_desc` when adding to GameData.

- `AmethystApplication::with_setup` runs the function before the dispatcher.

  **Quick fix:**

  - Find: `with_setup`,
  - Replace: `with_effect`

- Renamed `UiTransformBuilder` to `UiTransformData`.

- Renamed `UiTextBuilder` to `UiTextData`.

- Renamed `UiButtonBuilder` to `UiButtonData`.
