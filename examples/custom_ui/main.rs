//! Custom UI example

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    audio::AudioFormat,
    core::transform::TransformBundle,
    prelude::*,
    renderer::{DrawShaded, PosNormTex, TextureFormat},
    ui::{FontFormat, ToNativeWidget, UiBundle, UiCreator, UiTransformBuilder, UiWidget},
    utils::{application_root_dir, scene::BasicScenePrefab},
};

use serde::Deserialize;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

#[derive(Clone, Deserialize)]
enum CustomUi {
    // Example widget which repeats its `item`
    Repeat {
        x_move: f32,
        y_move: f32,
        count: usize,
        item: UiWidget<AudioFormat, TextureFormat, FontFormat, CustomUi>,
    },
}

impl ToNativeWidget for CustomUi {
    type PrefabData = ();
    fn to_native_widget(
        self,
        _: (),
    ) -> (
        UiWidget<AudioFormat, TextureFormat, FontFormat, CustomUi>,
        Self::PrefabData,
    ) {
        match self {
            CustomUi::Repeat {
                count,
                item,
                x_move,
                y_move,
            } => {
                let transform = item
                    .transform()
                    .cloned()
                    .unwrap_or_else(|| Default::default());
                let mut pos = (0., 0., 100.);
                let children = std::iter::repeat(item)
                    .map(|widget| {
                        let widget = UiWidget::Container {
                            background: None,
                            transform: UiTransformBuilder::default()
                                .with_position(pos.0, pos.1, pos.2),
                            children: vec![widget],
                        };
                        pos.0 += x_move;
                        pos.1 += y_move;
                        widget
                    })
                    .take(count)
                    .collect();
                let widget = UiWidget::Container {
                    background: None,
                    transform,
                    children,
                };
                (widget, ())
            }
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        let handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        world.create_entity().with(handle).build();

        // Load custom UI prefab
        world.exec(
            |mut creator: UiCreator<'_, AudioFormat, TextureFormat, FontFormat, CustomUi>| {
                creator.create("ui/custom.ron", ());
            },
        );
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/custom_ui/resources/display.ron");
    let resources = app_root.join("examples/assets");

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String, CustomUi>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
