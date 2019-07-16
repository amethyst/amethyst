# Rendy: Migration Guide

## Audio

* `AudioFormat` no longer exists, you have to use the lower level types -- `Mp3Format`, `WavFormat`, `OggFormat`, `FlacFormat`.

## Assets

* `SimpleFormat` trait has merged into `Format`.
* `Format::Options` associated type has been removed; options are now stored in the format instance.
* `NAME` associated constant is now a method call.
* `Format<A>` type parameter now takes in `Format<D>`, where `D` is `A::Data`.
* Implement `import_simple` instead of `import`.
* `Loader::load` no longer takes in the `Options` parameter.

## Input

* `Bindings<String, String>` is now `Bindings<StringBindings>`.
* `Bindings<AX, AC>` is now `Bindings<T>`, where `T` is a new type you must implement:

    ```rust,ignore
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

* `InputBundle` type parameters:

    ```patch
    -InputBundle::<String, String>::new()
    +InputBundle::<StringBindings>::new()
    ```

* `UiBundle` type parameters:

    ```patch
    +use amethyst::renderer::types::DefaultBackend;

    -UiBundle::<String, String>::new()
    +UiBundle::<DefaultBackend, StringBindings>::new()
    ```

## Window

* `DisplayConfig`'s `fullscreen` field is now an `Option<MonitorIdent>`. `MonitorIdent` is `MonitorIdent(u16, String)`, indicating the native monitor display ID, and its [name](https://docs.rs/winit/0.19.1/winit/struct.MonitorId.html#method.get_name).
* `WindowBundle` is now separate from `amethyst_renderer`.

    ```rust,ignore
    use amethyst::window::WindowBundle;

    game_data.with_bundle(WindowBundle::from_config_file(display_config_path))?;
    ```

## Renderer

* `amethyst::renderer::VirtualKeyCode` is now `amethyst::input::VirtualKeyCode`
* `amethyst::renderer::DisplayConfig` is now `amethyst::window::DisplayConfig`
* `amethyst::renderer::WindowEvent` is now `amethyst::winit::WindowEvent`
* `amethyst::renderer::Event` is no longer re-exported. Use `amethyst::winit::Event`
* `amethyst::renderer::Transparent` is now under `amethyst::renderer::transparent::Transparent`.
* `amethyst::renderer::Visibility` is now under `amethyst::renderer::visibility::Visibility`.
* `TextureHandle` type alias no longer exists, use `Handle<Texture>`.
* `Flipped` component is removed. You can specify `flipped` during sprite loading, or mutating `Transform` at run time.
* To load a texture in memory, you can't use `[0.; 4].into()` as the `TextureData` anymore. Use:

    ```rust,ignore
    use amethyst::{
        assets::{AssetStorage, Handle, Loader, Prefab, PrefabLoader},
        ecs::World,
        renderer::{
            loaders::load_from_srgba,
            palette::Srgba,
            types::TextureData,
            Texture,
        },
    };

    let loader = world.read_resource::<Loader>();
    let texture_assets = world.read_resource::<AssetStorage<Texture>>();
    let texture_builder = load_from_srgba(Srgba::new(0., 0., 0., 0.));
    let texture_handle: Handle<Texture> =
        loader.load_from_data(TextureData::from(texture_builder), (), &texture_assets);
    ```

* `RenderBundle` and `Pipeline` are gone, now you need to create a `RenderGraph`, for example:

    In `main.rs`:

    ```rust,ignore
    use amethyst::renderer::{types::DefaultBackend, RenderingSystem};

    use crate::render_graph::RenderGraph;

    mod render_graph;

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config(display_config))?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderGraph::default(),
        ));
    ```

    In `render_graph.rs`:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    use amethyst::{
        ecs::{ReadExpect, Resources, SystemData},
        renderer::{
            pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc},
            rendy::{
                factory::Factory,
                graph::{
                    present::PresentNode,
                    render::{RenderGroupDesc, SubpassBuilder},
                    GraphBuilder,
                },
                hal::{
                    command::{ClearDepthStencil, ClearValue},
                    format::Format,
                    image::Kind,
                },
            },
            types::DefaultBackend,
            GraphCreator,
        },
        ui::DrawUiDesc,
        window::{ScreenDimensions, Window},
    };

    #[derive(Default)]
    pub struct RenderGraph {
        dimensions: Option<ScreenDimensions>,
        dirty: bool,
    }

    impl GraphCreator<DefaultBackend> for RenderGraph {
        #[allow(clippy::map_clone)]
        fn rebuild(&mut self, res: &Resources) -> bool {
            // Rebuild when dimensions change, but wait until at least two frames have the same.
            let new_dimensions = res.try_fetch::<ScreenDimensions>();
            use std::ops::Deref;
            if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
                self.dirty = true;
                self.dimensions = new_dimensions.map(|d| d.clone());
                return false;
            }
            self.dirty
        }

        fn builder(
            &mut self,
            factory: &mut Factory<DefaultBackend>,
            res: &Resources,
        ) -> GraphBuilder<DefaultBackend, Resources> {
            self.dirty = false;

            let window = <ReadExpect<'_, Window>>::fetch(res);
            let dimensions = self.dimensions.as_ref().unwrap();
            let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);
            let surface = factory.create_surface(&window);
            let surface_format = factory.get_surface_format(&surface);

            let mut graph_builder = GraphBuilder::new();
            let color = graph_builder.create_image(
                window_kind,
                1,
                surface_format,
                // clear screen to black
                Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
            );

            let depth = graph_builder.create_image(
                window_kind,
                1,
                Format::D32Sfloat,
                Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
            );

            let sprite_pass = graph_builder.add_node(
                SubpassBuilder::new()
                    .with_group(DrawFlat2DDesc::new().builder())
                    .with_group(DrawFlat2DTransparentDesc::new().builder())
                    .with_color(color)
                    .with_depth_stencil(depth)
                    .into_pass(),
            );
            let ui_pass = graph_builder.add_node(
                SubpassBuilder::new()
                    .with_dependency(sprite_pass)
                    .with_group(DrawUiDesc::new().builder())
                    .with_color(color)
                    .with_depth_stencil(depth)
                    .into_pass(),
            );

            let _present = graph_builder.add_node(
                PresentNode::builder(factory, surface, color)
                    .with_dependency(sprite_pass)
                    .with_dependency(ui_pass),
            );

            graph_builder
        }
    }

    ```

* `RenderBundle::with_sprite_sheet_processor()` is replaced by:

    ```rust,ignore
    game_data.with(
        Processor::<SpriteSheet>::new(),
        "sprite_sheet_processor",
        &[],
    );
    ```

* `RenderBundle::with_sprite_visibility_sorting()` is replaced by:

    ```rust,ignore
    use amethyst::rendy::sprite_visibility::SpriteVisibilitySortingSystem;

    game_data.with(
        SpriteVisibilitySortingSystem::new(),
        "sprite_visibility_system",
        &["transform_system"],
    );
    ```

* Sprite transparency is no longer a separate flag. Instead of `with_transparency`, you add a second render pass using `DrawFlat2DTransparent`. See the [`sprites_ordered` example](https://github.com/amethyst/amethyst/blob/7ed8432d8eef2b2727d0c4188b91e5823ae03548/examples/sprites_ordered/main.rs#L463-L482).

Camera changes:

* `CameraPrefab` is no longer nested:

    ```patch
    -camera: Perspective((aspect: 1.3, fovy: 1.0471975512, znear: 0.1, zfar: 2000.0))
    +camera: Perspective(aspect: 1.3, fovy: 1.0471975512, znear: 0.1, zfar: 2000.0)
    ```

* `nalgebra`'s `Perspective3`/`Orthographic3` are *no longer compatible*, as they use OpenGL coordinates instead of Vulkan.

    Amethyst now has amethyst::rendy::camera::Orthographic and Perspective, respectively. These types are mostly feature-parity with nalgebra, but correct for vulkan. You can use as_matrix to get the inner Matrix4 value.

* Camera now stores `Projection`, which means it is type-safe.
* You can no longer serialize raw camera matrices, only camera parameter types.

Z-axis direction clarifications:

* In Vulkan, `Z+` is away.
* in OpenGL, `Z-` is away.
* In amethyst_renderer, `Z-` is away (world coordinates).
* In amethyst_rendy, `Z-` is away (world coordinates).

## Amethyst Test

* The `render_base` function has been changed:

    ```patch
    -let visibility = false;
    -AmethystApplication::render_base("test_name", visibility);
    +use amethyst::renderer::{types::DefaultBackend, RenderEmptyBundle};
    +AmethystApplication::blank()
    +    .with_bundle(RenderEmptyBundle::<DefaultBackend>::new());
    ```

* The `mark_render()` and `.run()` chained call is replaced by a single `run_isolated()` call.
