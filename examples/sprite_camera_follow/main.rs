use std::sync::Arc;
use amethyst::{
    assets::{AssetStorage, Loader, Processor},
    core::{Parent, Transform, TransformBundle},
    ecs::{Component, Entity, Join, NullStorage, Read, ReadStorage, System, WriteStorage, ReadExpect, SystemData, Resources},
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        pass::DrawFlat2DTransparentDesc,
        camera::Camera,
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::format::Format,
        },
        formats::texture::ImageFormat,
        Texture,
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
        sprite::{SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        sprite_visibility::{SpriteVisibilitySortingSystem},
        transparent::Transparent,
    },
    window::{WindowBundle, Window, ScreenDimensions},
    utils::application_root_dir,
};

#[derive(Default)]
struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, mut transforms, cameras, input,): Self::SystemData) {
        let x_move = input.axis_value("entity_x").unwrap();
        let y_move = input.axis_value("entity_y").unwrap();

        for (_, transform) in (&players, &mut transforms).join() {
            transform.prepend_translation_x(x_move as f32 * 5.0);
            transform.prepend_translation_y(y_move as f32 * 5.0);
            println!("Player = {:?}", transform);
        }

        for (_, transform) in (&cameras, &mut transforms).join() {
            transform.prepend_translation_x(x_move as f32 * 5.0);
            transform.prepend_translation_y(y_move as f32 * 5.0);
            println!("Camera = {:?}", transform);
        }
    }
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

// Initialize a background
fn init_background_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_z(-10.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world.create_entity().with(transform).with(sprite).build()
}

// Initialize a sprite as a reference point at a fixed location
fn init_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .build()
}

// Initialize a sprite as a reference point
fn init_screen_reference_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(10.0, 10.0, 0.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world
        .create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn init_player(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 1,
    };
    world
        .create_entity()
        .with(transform)
        .with(Player)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn initialise_camera(world: &mut World, parent: Entity) -> Entity {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    //println!("Init camera with dimensions: {}x{}", width, height);

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Parent{entity: parent})
        .with(Camera::standard_2d(width, height))
        .build()
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let circle_sprite_sheet_handle =
            load_sprite_sheet(world, "Circle_Spritesheet.png", "Circle_Spritesheet.ron");
        let background_sprite_sheet_handle =
            load_sprite_sheet(world, "Background.png", "Background.ron");

        //let _background = init_background_sprite(world, &background_sprite_sheet_handle);
        let _reference = init_reference_sprite(world, &circle_sprite_sheet_handle);
        let player = init_player(world, &circle_sprite_sheet_handle);
        let _camera = initialise_camera(world, player);
        let _reference_screen = init_screen_reference_sprite(world, &circle_sprite_sheet_handle);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/sprite_camera_follow/resources");
    let display_config_path = app_root.join("examples/sprite_camera_follow/resources/display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(assets_directory.join("input.ron"))?,
        )?
        .with(MovementSystem, "movement", &[])
        .with(Processor::<SpriteSheet>::new(), "sprite_sheet_processor", &[], )
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &["transform_system"],
        )
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(ExampleGraph::new()));

    let mut game = Application::build(assets_directory, Example)?.build(game_data)?;
    game.run();
    Ok(())
}



struct ExampleGraph {
    last_dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}

impl ExampleGraph {
    pub fn new() -> Self {
        Self {
            last_dimensions: None,
            surface_format: None,
            dirty: true,
        }
    }
}

impl GraphCreator<DefaultBackend> for ExampleGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.last_dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.last_dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);
        let surface = factory.create_surface(window.clone());
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            surface.kind(),
            1,
            surface_format,
            Some(ClearValue::Color([0.34, 0.36, 0.52, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            surface.kind(),
            1,
            Format::D32Float,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DTransparentDesc::default().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(sprite));

        graph_builder
    }
}