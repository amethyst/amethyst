use amethyst::{
    renderer::{types::DefaultBackend, RenderToWindow, RenderingBundle},
    ui::AudioUiBundle,
    Application, GameData, LoggerConfig, SimpleState, StateData,
};
use amethyst_assets::AssetProcessorSystemBundle;
use amethyst_audio::{output::init_output, Source};
use amethyst_core::{dispatcher::DispatcherBuilder, transform::TransformBundle};
use amethyst_input::InputBundle;
use amethyst_ui::{RenderUi, UiBundle};
use amethyst_utils::application_root_dir;

#[derive(Default)]
struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        example_utils::build_example_button(world, resources);
        example_utils::build_ui_image_texture(world, resources);

        // We init the output because complex button has sounds
        init_output(resources);
        example_utils::build_complex_button_with_font_and_sound(world, resources);

        example_utils::build_draggable(world, resources);
        example_utils::build_multi_line_label(world, resources);
        example_utils::build_editable_text(world, resources);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(LoggerConfig::default());
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/ui/config/display.ron");
    let assets_dir = app_root.join("examples/ui/assets");

    let mut dispatcher = DispatcherBuilder::default();

    dispatcher
        .add_bundle(TransformBundle::default())
        .add_bundle(InputBundle::default())
        .add_bundle(AssetProcessorSystemBundle::<Source>::default())
        .add_bundle(UiBundle::<u32>::default())
        .add_bundle(AudioUiBundle::default())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderUi::default()),
        );

    let mut game = Application::new(assets_dir, Example::default(), dispatcher)?;
    game.run();
    Ok(())
}

pub struct TestCpnt;

mod example_utils {
    use amethyst::ecs::{Resources, World};
    use amethyst_assets::{AssetStorage, Format, Loader};
    use amethyst_audio::{OggFormat, Source};
    use amethyst_rendy::{ImageFormat, Texture};
    use amethyst_ui::{
        Anchor, Draggable, FontAsset, Interactable, LineMode, TextEditing, TtfFormat,
        UiButtonBuilder, UiImage, UiLabelBuilder, UiTransform,
    };

    pub fn build_example_button(world: &mut World, resources: &mut Resources) {
        UiButtonBuilder::<(), u32>::new("Made with UiButtonBuilder".to_string())
            .with_font_size(32.0)
            .with_position(0.0, -256.0)
            .with_size(64.0 * 6.0, 64.0)
            .with_anchor(Anchor::TopMiddle)
            .with_image(UiImage::SolidColor([0.8, 0.6, 0.3, 1.0]))
            .with_hover_image(UiImage::SolidColor([0.1, 0.1, 0.1, 0.5]))
            .with_text_color([1., 0., 0., 1.])
            .with_hover_text_color([1., 1., 1., 1.])
            .build_from_world_and_resources(world, resources);
    }

    pub fn build_multi_line_label(world: &mut World, resources: &mut Resources) {
        let font = {
            let font_storage = resources.get_mut::<AssetStorage<FontAsset>>().unwrap();
            resources
                .get::<Loader>()
                .unwrap()
                .load("font/square.ttf", TtfFormat, (), &font_storage)
        };
        UiLabelBuilder::<(), u32>::new("Multiline\nText!")
            .with_line_mode(LineMode::Wrap)
            .with_position(-200., 0.)
            .with_size(400., 200.)
            .with_anchor(Anchor::MiddleRight)
            .with_font(font)
            .with_font_size(30.)
            .with_text_color([0.2, 0.2, 1.0, 1.0])
            .with_align(Anchor::MiddleRight)
            .build_from_world_and_resources(world, resources);
    }

    pub fn build_editable_text(world: &mut World, resources: &mut Resources) {
        let font = {
            let font_storage = resources.get_mut::<AssetStorage<FontAsset>>().unwrap();
            resources
                .get::<Loader>()
                .unwrap()
                .load("font/square.ttf", TtfFormat, (), &font_storage)
        };

        let text = UiLabelBuilder::<(), u32>::new("Editable")
            .with_line_mode(LineMode::Single)
            .with_position(270., 50.)
            .with_size(500., 75.)
            .with_layer(10.0)
            .with_anchor(Anchor::BottomLeft)
            .with_font(font)
            .with_font_size(75.)
            .with_text_color([0.2, 0.2, 1.0, 1.0])
            .with_align(Anchor::BottomLeft)
            .with_selectable(1)
            .build_from_world_and_resources(world, resources);

        let editing = TextEditing::new(2000, [0.09, 0.02, 0.25, 1.0], [1.0, 0.5, 0.8, 1.0], false);

        world
            .entry(text.1.text_entity)
            .unwrap()
            .add_component(editing);
        world
            .entry(text.1.text_entity)
            .unwrap()
            .add_component(Interactable);
    }

    pub fn build_complex_button_with_font_and_sound(world: &mut World, resources: &mut Resources) {
        let font = {
            let font_storage = resources.get_mut::<AssetStorage<FontAsset>>().unwrap();
            resources
                .get::<Loader>()
                .unwrap()
                .load("font/square.ttf", TtfFormat, (), &font_storage)
        };

        let hover_sound = {
            let sound_storage = resources.get_mut::<AssetStorage<Source>>().unwrap();
            resources
                .get::<Loader>()
                .unwrap()
                .load("audio/boop.ogg", OggFormat, (), &sound_storage)
        };

        let confirm_sound = {
            let sound_storage = resources.get_mut::<AssetStorage<Source>>().unwrap();
            resources.get::<Loader>().unwrap().load(
                "audio/confirm.ogg",
                OggFormat,
                (),
                &sound_storage,
            )
        };

        UiButtonBuilder::<(), u32>::new("ComplexBtn".to_string())
            .with_font_size(20.0)
            .with_position(0.0, -32.0)
            .with_size(128., 64.0)
            .with_anchor(Anchor::TopMiddle)
            .with_tab_order(9)
            .with_font_size(20.)
            .with_text_color([0.2, 0.2, 1.0, 1.0])
            .with_hover_text_color([0.4, 0.4, 1.0, 1.0])
            .with_press_text_color([0.6, 0.6, 1.0, 1.0])
            .with_image(UiImage::SolidColor([0., 1., 0., 1.]))
            .with_hover_image(UiImage::SolidColor([0.3, 1., 0.3, 1.]))
            .with_press_image(UiImage::SolidColor([0.15, 1., 0.15, 1.]))
            .with_font(font)
            .with_hover_sound(hover_sound)
            .with_press_sound(confirm_sound)
            .build_from_world_and_resources(world, resources);
    }

    pub fn build_ui_image_texture(world: &mut World, resources: &mut Resources) {
        let loader = resources.get::<Loader>().unwrap();
        let texture_storage = resources.get_mut::<AssetStorage<Texture>>().unwrap();
        let texture_data = loader.load_from_data(
            ImageFormat::default()
                .import_simple(include_bytes!("assets/texture/logo_transparent.png").to_vec())
                .expect("Unable to read logo image'"),
            (),
            &texture_storage,
        );

        let image = UiImage::Texture(texture_data);
        let transform = UiTransform::new(
            String::from("logo"),
            Anchor::BottomMiddle,
            Anchor::Middle,
            0.,
            32.,
            1.,
            64.,
            64.,
        );
        world.push((image, transform));
    }

    pub fn build_draggable(world: &mut World, resources: &mut Resources) {
        let font = {
            let font_storage = resources.get_mut::<AssetStorage<FontAsset>>().unwrap();
            resources
                .get::<Loader>()
                .unwrap()
                .load("font/square.ttf", TtfFormat, (), &font_storage)
        };
        let (_, btn) = UiButtonBuilder::<(), u32>::new("Draggable".to_string())
            .with_font_size(20.0)
            .with_position(250., -120.)
            .with_layer(1.)
            .with_size(128., 64.0)
            .with_tab_order(15)
            .with_anchor(Anchor::TopLeft)
            .with_font(font)
            .with_text_color([0.0, 0.0, 0.0, 1.0])
            .with_image(UiImage::SolidColor([0.82, 0.83, 0.83, 1.]))
            .build_from_world_and_resources(world, resources);

        world
            .entry(btn.image_entity)
            .unwrap()
            .add_component(Draggable);
    }
}
