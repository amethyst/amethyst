//! Custom UI example

#[macro_use]
extern crate amethyst;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    audio::AudioFormat,
    core::transform::TransformBundle,
    prelude::*,
    renderer::{DrawShaded, PosNormTex, TextureFormat},
    ui::{FontFormat, ToNativeWidget, UiBundle, UiCreator, UiTransformBuilder, UiWidget},
    utils::{application_root_dir, scene::BasicScenePrefab},
};

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
                    }).take(count)
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

#[derive(State, Debug, Clone)]
enum State {
    Example,
}

struct Example;

impl<S, E> StateCallback<S, E> for Example {
    fn on_start(&mut self, world: &mut World) {
        // Initialise the scene with an object, a light and a camera.
        let handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        world.create_entity().with(handle).build();

        // Load custom UI prefab
        world.exec(
            |mut creator: UiCreator<AudioFormat, TextureFormat, FontFormat, CustomUi>| {
                creator.create("ui/custom.ron", ());
            },
        );
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let display_config_path = format!("{}/examples/ui/resources/display.ron", app_root);

    let resources = format!("{}/examples/assets", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String, CustomUi>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;

    let mut game = Application::build(resources)?
        .with_defaults()
        .with_state(State::Example, Example)?
        .build(game_data)?;

    game.run();
    Ok(())
}
