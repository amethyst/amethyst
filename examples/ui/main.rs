//! Displays a shaded sphere to the user.

extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::{GlobalTransform, TransformBundle};
use amethyst::core::Time;
use amethyst::ecs::prelude::{Entity, System, World, Write};
use amethyst::input::{is_close_requested, is_key, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{AmbientColor, Camera, DrawShaded, Light, Mesh, PngFormat, PointLight,
                         PosNormTex, Projection, Rgba, Texture, Shape};
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::ui::{Anchor, FontAsset, MouseReactive, Stretch, TextEditing, TtfFormat,
                   UiBundle, UiButtonBuilder, UiEvent, UiFocused, UiImage,
                   UiText, UiTransform};
use amethyst::utils::fps_counter::{FPSCounter, FPSCounterBundle};
use amethyst::winit::{Event, VirtualKeyCode};

const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // blue
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0); // near-black
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0); // white
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

struct Example {
    fps_display: Option<Entity>,
}

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        initialise_sphere(world);
        initialise_lights(world);
        initialise_camera(world);
        let (logo, font, background_color, green) = {
            let loader = world.read_resource::<Loader>();

            let logo = loader.load(
                "texture/logo_transparent.png",
                PngFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );

            let font = loader.load(
                "font/square.ttf",
                TtfFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<FontAsset>>(),
            );
            let background_color = loader.load_from_data(
                [0.36, 0.10, 0.57, 1.0].into(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );
            let green = loader.load_from_data(
                [0.0, 1.0, 0.0, 1.0].into(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );
            (logo, font, background_color, green)
        };

        let background = world
            .create_entity()
            .with(
                UiTransform::new(
                    "background".to_string(),
                    Anchor::Middle,
                    0.0,
                    0.0,
                    10.0,
                    20.0,
                    20.0,
                    0,
                ).with_stretching(Stretch::XY {
                    x_margin: 0.0,
                    y_margin: 0.0,
                }),
            )
            .with(UiImage {
                texture: background_color.clone(),
            })
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "logo".to_string(),
                Anchor::BottomMiddle,
                0.,
                32.,
                -3.,
                64.,
                64.,
                1,
            ))
            .with(UiImage {
                texture: logo.clone(),
            })
            .with(MouseReactive)
            .build();

        let text = world
            .create_entity()
            .with(UiTransform::new(
                "hello_world".to_string(),
                Anchor::Middle,
                0.,
                0.,
                -4.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "Hello world!".to_string(),
                [0.5, 0.5, 1.0, 1.0],
                75.,
            ))
            .with(TextEditing::new(
                12,
                [0.0, 0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
                false,
            ))
            .build();

        UiButtonBuilder::new("btn_transform", "Button!")
            .with_font(font.clone())
            .with_text_color([0.2, 0.2, 1.0, 1.0])
            .with_font_size(20.)
            .with_position(0.0, 32.0)
            .with_layer(-1.)
            .with_size(128., 64.)
            .with_tab_order(9)
            .with_image(green.clone())
            .with_anchor(Anchor::TopMiddle)
            .with_parent(background.clone())
            .build_from_world(world);

        UiButtonBuilder::new("simple_btn", "Simpler!")
            .with_font(font.clone())
            .with_position(250.0, 50.0)
            .build_from_world(world);

        let fps = world
            .create_entity()
            .with(UiTransform::new(
                "fps".to_string(),
                Anchor::TopLeft,
                100.,
                30.,
                -3.,
                500.,
                75.,
                2,
            ))
            .with(UiText::new(
                font,
                "N/A".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                75.,
            ))
            .build();
        self.fps_display = Some(fps);
        world.write_resource::<UiFocused>().entity = Some(text);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, state_data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        let StateData { world, data } = state_data;
        data.update(&world);
        let mut ui_text = world.write_storage::<UiText>();
        if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FPSCounter>().sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }

        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    let display_config_path = format!(
        "{}/examples/ui/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with(UiEventHandlerSystem::new(), "ui_event_handler", &[])
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, Example { fps_display: None }, game_data)?;
    game.run();
    Ok(())
}

/// This function initialises a sphere and adds it to the world.
fn initialise_sphere(world: &mut World) {
    // Create a sphere mesh and material.

    use amethyst::assets::Handle;
    use amethyst::renderer::{Material, MaterialDefaults};

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh: Handle<Mesh> = loader.load_from_data(
            Shape::Sphere(32, 32).generate::<Vec<PosNormTex>>(None),
            (),
            &world.read_resource(),
        );

        let albedo = SPHERE_COLOUR.into();

        let tex_storage = world.read_resource();
        let mat_defaults = world.read_resource::<MaterialDefaults>();

        let albedo = loader.load_from_data(albedo, (), &tex_storage);

        let mat = Material {
            albedo,
            ..mat_defaults.0.clone()
        };

        (mesh, mat)
    };

    // Create a sphere entity using the mesh and the material.
    world
        .create_entity()
        .with(GlobalTransform::default())
        .with(mesh)
        .with(material)
        .build();
}

/// This function adds an ambient light and a point light to the world.
fn initialise_lights(world: &mut World) {
    // Add ambient light.
    world.add_resource(AmbientColor(AMBIENT_LIGHT_COLOUR));

    let light: Light = PointLight {
        center: LIGHT_POSITION.into(),
        radius: LIGHT_RADIUS,
        intensity: LIGHT_INTENSITY,
        color: POINT_LIGHT_COLOUR,
        ..Default::default()
    }.into();

    // Add point light.
    world.create_entity().with(light).build();
}

/// This function initialises a camera and adds it to the world.
fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::Matrix4;
    let transform =
        Matrix4::from_translation([0.0, 0.0, -4.0].into()) * Matrix4::from_angle_y(Deg(180.));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}

/// This shows how to handle UI events.
pub struct UiEventHandlerSystem {
    reader_id: Option<ReaderId<UiEvent>>,
}

impl UiEventHandlerSystem {
    pub fn new() -> Self {
        UiEventHandlerSystem { reader_id: None }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = Write<'a, EventChannel<UiEvent>>;

    fn run(&mut self, mut events: Self::SystemData) {
        if self.reader_id.is_none() {
            self.reader_id = Some(events.register_reader());
        }
        for ev in events.read(self.reader_id.as_mut().unwrap()) {
            info!("You just interacted with a ui element: {:?}", ev);
        }
    }
}
