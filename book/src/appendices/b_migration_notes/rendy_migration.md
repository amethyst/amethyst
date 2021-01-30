# Rendy: Migration Guide

## Audio

- `AudioFormat` no longer exists, you have to use the lower level types -- `Mp3Format`, `WavFormat`, `OggFormat`, `FlacFormat`.

## Assets

- `SimpleFormat` trait has merged into `Format`.
- `Format::Options` associated type has been removed; options are now stored in the format instance.
- `NAME` associated constant is now a method call.
- `Format<A>` type parameter now takes in `Format<D>`, where `D` is `A::Data`.
- Implement `import_simple` instead of `import`.
- `Loader::load` no longer takes in the `Options` parameter.

## Input

- `Bindings<String, String>` is now `Bindings<StringBindings>`.

- `Bindings<AX, AC>` is now `Bindings<T>`, where `T` is a new type you must implement:

  ```rust ,ignore
  pub struct ControlBindings;

  impl BindingTypes for ControlBindings {
      type Axis = PlayerAxisControl;
      type Action = PlayerActionControl;
  }
  ```

  Diff:

  ```patch
  -Bindings<PlayerAxisControl, PlayerActionControl>
  +Bindings<ControlBindings>
  ```

- `InputBundle` type parameters:

  ```patch
  -InputBundle::<String, String>::new()
  +InputBundle::new()
  ```

- `UiBundle` type parameters:

  ```patch
  +use amethyst::renderer::types::DefaultBackend;

  -UiBundle::<String, String>::new()
  +UiBundle::<DefaultBackend>::new()
  ```

## Window

- `DisplayConfig`'s `fullscreen` field is now an `Option<MonitorIdent>`. `MonitorIdent` is `MonitorIdent(u16, String)`, indicating the native monitor display ID, and its [name][monid].

- `WindowBundle` is now separate from `amethyst_renderer`.

  ```rust ,ignore
  use amethyst::window::WindowBundle;

  game_data.add_bundle(WindowBundle::from_config_file(display_config_path))?;
  ```

  This system is loaded automatically by the `RenderToWindow` render plugin.

## Renderer

- `amethyst::renderer::VirtualKeyCode` is now `amethyst::input::VirtualKeyCode`

- `amethyst::renderer::DisplayConfig` is now `amethyst::window::DisplayConfig`

- `amethyst::renderer::WindowEvent` is now `amethyst::winit::WindowEvent`

- `amethyst::renderer::Event` is no longer re-exported. Use `amethyst::winit::Event`

- `amethyst::renderer::Transparent` is now under `amethyst::renderer::transparent::Transparent`.

- `amethyst::renderer::Visibility` is now under `amethyst::renderer::visibility::Visibility`.

- `TextureHandle` type alias no longer exists, use `Handle<Texture>`.

- `Flipped` component is removed. You can specify `flipped` during sprite loading, or mutating `Transform` at run time.

- To load a texture in memory, you can't use `[0.; 4].into()` as the `TextureData` anymore. Use:

  ```rust ,ignore
  use amethyst::{
      assets::{AssetStorage, Handle,  DefaultLoader, Loader, Prefab, PrefabLoader},
      ecs::World,
      renderer::{
          loaders::load_from_srgba,
          palette::Srgba,
          types::TextureData,
          Texture,
      },
  };

  let loader = world.read_resource::<DefaultLoader>();
  let texture_assets = world.read_resource::<AssetStorage<Texture>>();
  let texture_builder = load_from_srgba(Srgba::new(0., 0., 0., 0.));
  let texture_handle: Handle<Texture> =
      loader.load_from_data(TextureData::from(texture_builder), (), &texture_assets);
  ```

- `RenderBundle` and `Pipeline` are gone, now you need to use the `RenderingBundle`, for example:

  In `main.rs`:

  ```rust ,ignore
  use amethyst::renderer::{types::DefaultBackend, RenderingSystem};

  let game_data = DispatcherBuilder::default()
      .add_bundle(
          RenderingBundle::<DefaultBackend>::new()
              .with_plugin(
                  RenderToWindow::from_config_path(display_config)
                      .with_clear([0.34, 0.36, 0.52, 1.0]),
              )
              .with_plugin(RenderShaded3D::default())
              .with_plugin(RenderDebugLines::default())
              .with_plugin(RenderSkybox::with_colors(
                  Srgb::new(0.82, 0.51, 0.50),
                  Srgb::new(0.18, 0.11, 0.85),
              )),
      )?;
  ```

- Render passes can be integrated into amethyst by using the newly introduced `RenderPlugin` trait, for example:

  ```rust ,ignore
  pub struct RenderCustom {
      target: Target,
  }

  impl RenderTerrain {
      /// Set target to which 2d sprites will be rendered.
      pub fn with_target(mut self, target: Target) -> Self {
          self.target = target;
          self
      }
  }

  impl<B: Backend> RenderPlugin<B> for RenderCustom {
      fn on_build<'a, 'b>(&mut self, builder: &mut DispatcherBuilder) -> Result<(), Error> {
          // You can add systems that are needed by your renderpass here
          Ok(())
      }

      fn on_plan(
          &mut self,
          plan: &mut RenderPlan<B>,
          _factory: &mut Factory<B>,
          _res: &Resources,
      ) -> Result<(), Error> {
          plan.extend_target(self.target, |ctx| {
              ctx.add(RenderOrder::Opaque, DrawCustomDesc::new().builder())?;
              Ok(())
          });
          Ok(())
      }
  }
  ```

- `RenderBundle::with_sprite_sheet_processor()` is replaced by:

  ```rust ,ignore
  game_data.with(
      Processor::<SpriteSheet>::new(),
      "sprite_sheet_processor",
      &[],
  );
  ```

  This system is added automatically by each of the 3D render plugins (`RenderPbr3D`, `RenderShaded3D`, `RenderFlat3D`).

- `RenderBundle::with_sprite_visibility_sorting()` is replaced by:

  ```rust ,ignore
  use amethyst::rendy::sprite_visibility::SpriteVisibilitySortingSystem;

  game_data.with(
      SpriteVisibilitySortingSystem::new(),
      "sprite_visibility_system",
      &["transform_system"],
  );
  ```

  This system is added automatically by the `RenderFlat2D` render plugin.

- Sprite transparency is no longer a separate flag. Instead of `with_transparency`, you add a second render pass using `DrawFlat2DTransparent`. See the [`sprites_ordered` example][spri_ord].

Camera changes:

- `CameraPrefab` is no longer nested:

  ```patch
  -camera: Perspective((aspect: 1.3, fovy: 1.0471975512, znear: 0.1, zfar: 2000.0))
  +camera: Perspective(aspect: 1.3, fovy: 1.0471975512, znear: 0.1, zfar: 2000.0)
  ```

- `nalgebra`'s `Perspective3`/`Orthographic3` are *no longer compatible*, as they use OpenGL coordinates instead of Vulkan.

  Amethyst now has amethyst::rendy::camera::Orthographic and Perspective, respectively. These types are mostly feature-parity with nalgebra, but correct for vulkan. You can use as\_matrix to get the inner Matrix4 value.

- Camera now stores `Projection`, which means it is type-safe.

- You can no longer serialize raw camera matrices, only camera parameter types.

Z-axis direction clarifications:

- In Vulkan, `Z+` is away.
- in OpenGL, `Z-` is away.
- In amethyst\_renderer, `Z-` is away (world coordinates).
- In amethyst\_rendy, `Z-` is away (world coordinates).

## Amethyst Test

- The `render_base` function has been changed:

  ```patch
  -let visibility = false;
  -AmethystApplication::render_base("test_name", visibility);
  +use amethyst::renderer::{types::DefaultBackend, RenderEmptyBundle};
  +AmethystApplication::blank()
  +    .add_bundle(RenderEmptyBundle::<DefaultBackend>::new());
  ```

- The `mark_render()` and `.run()` chained call is replaced by a single `run_isolated()` call.

[monid]: https://docs.rs/winit/0.19.1/winit/struct.MonitorId.html#method.get_name
[spri_ord]: https://github.com/amethyst/amethyst/blob/7ed8432d8eef2b2727d0c4188b91e5823ae03548/examples/sprites_ordered/main.rs#L463-L482
