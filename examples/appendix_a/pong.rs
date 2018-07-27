use amethyst::assets::Loader;
use amethyst::core::cgmath::Vector3;
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::ecs::prelude::World;
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, Event, Material, MeshHandle, PosTex, Projection, VirtualKeyCode, WindowMessages,
};
use amethyst::ui::{Anchor, TtfFormat, UiText, UiTransform};
use config::{ArenaConfig, BallConfig, PaddlesConfig};
use systems::ScoreText;
use {Ball, Paddle, Side};

pub struct Pong;

impl<'a, 'b> State<GameData<'a, 'b>> for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        use audio::initialise_audio;

        // Setup our game.
        initialise_paddles(world);
        initialise_balls(world);
        initialise_camera(world);
        initialise_audio(world);
        initialise_score(world);
        hide_cursor(world);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::{Matrix4, Vector3};
    let (arena_height, arena_width) = {
        let config = &world.read_resource::<ArenaConfig>();
        (config.height, config.width)
    };
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            arena_width,
            arena_height,
            0.0,
        )))
        .with(GlobalTransform(
            Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into(),
        ))
        .build();
}

/// Hide the cursor, so it's invisible while playing.
fn hide_cursor(world: &mut World) {
    use amethyst::winit::CursorState;

    world
        .write_resource::<WindowMessages>()
        .send_command(|win| {
            if let Err(err) = win.set_cursor_state(CursorState::Hide) {
                eprintln!("Unable to make cursor hidden! Error: {:?}", err);
            }
        });
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    let (arena_height, arena_width) = {
        let config = &world.read_resource::<ArenaConfig>();
        (config.height, config.width)
    };
    let (
        left_height,
        left_width,
        left_velocity,
        left_colour,
        right_height,
        right_width,
        right_velocity,
        right_colour,
    ) = {
        let config = &world.read_resource::<PaddlesConfig>();
        let cl: [f32; 4] = [
            config.left.colour.0,
            config.left.colour.1,
            config.left.colour.2,
            config.left.colour.3,
        ];
        let cr: [f32; 4] = [
            config.right.colour.0,
            config.right.colour.1,
            config.right.colour.2,
            config.right.colour.3,
        ];
        (
            config.left.height,
            config.left.width,
            config.left.velocity,
            cl,
            config.right.height,
            config.right.width,
            config.right.velocity,
            cr,
        )
    };

    let left_y = (arena_height - left_height) / 2.0;
    let right_y = (arena_height - right_height) / 2.0;
    left_transform.translation = Vector3::new(0.0, left_y, 0.0);
    right_transform.translation = Vector3::new(arena_width - right_width, right_y, 0.0);

    let left_mesh = create_mesh(
        world,
        generate_rectangle_vertices(0.0, 0.0, left_width, left_height),
    );

    let right_mesh = create_mesh(
        world,
        generate_rectangle_vertices(0.0, 0.0, right_width, right_height),
    );
    let left_material = create_colour_material(world, left_colour);
    let right_material = create_colour_material(world, right_colour);
    // Create left paddle
    world
        .create_entity()
        .with(left_mesh)
        .with(left_material)
        .with(Paddle {
            side: Side::Left,
            height: left_height,
            width: left_width,
            velocity: left_velocity,
        })
        .with(GlobalTransform::default())
        .with(left_transform)
        .build();
    // Create right paddle
    world
        .create_entity()
        .with(right_mesh)
        .with(right_material)
        .with(Paddle {
            side: Side::Right,
            height: right_height,
            width: right_width,
            velocity: right_velocity,
        })
        .with(GlobalTransform::default())
        .with(right_transform)
        .build();
}

/// Initialises one ball in the middle-ish of the arena.
fn initialise_balls(world: &mut World) {
    let (arena_width, arena_height) = {
        let config = world.read_resource::<ArenaConfig>();
        (config.width, config.height)
    };
    let (velocity_x, velocity_y, radius, colour) = {
        let config = world.read_resource::<BallConfig>();
        let c: [f32; 4] = [
            config.colour.0,
            config.colour.1,
            config.colour.2,
            config.colour.3,
        ];
        (config.velocity.x, config.velocity.y, config.radius, c)
    };
    // Create the mesh, material and translation.
    let mesh = create_mesh(world, generate_circle_vertices(radius, 16));
    let material = create_colour_material(world, colour);
    let mut local_transform = Transform::default();
    local_transform.translation = Vector3::new(arena_width / 2.0, arena_height / 2.0, 0.0);

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(Ball {
            radius: radius,
            velocity: [velocity_x, velocity_y],
        })
        .with(local_transform)
        .with(GlobalTransform::default())
        .build();
}

fn initialise_score(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        Default::default(),
        (),
        &world.read_resource(),
    );
    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        -50.,
        50.,
        1.,
        55.,
        50.,
        0,
    );

    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        50.,
        50.,
        1.,
        55.,
        50.,
        0,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        ))
        .build();
    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        ))
        .build();
    world.add_resource(ScoreText { p1_score, p2_score });
}

/// Converts a vector of vertices into a mesh.
fn create_mesh(world: &World, vertices: Vec<PosTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), (), &world.read_resource())
}

/// Creates a solid material of the specified colour.
fn create_colour_material(world: &World, colour: [f32; 4]) -> Material {
    // TODO: optimize

    use amethyst::renderer::MaterialDefaults;

    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let albedo = loader.load_from_data(colour.into(), (), &world.read_resource());

    Material {
        albedo,
        ..mat_defaults.0.clone()
    }
}

/// Generates vertices for a circle. The circle will be made of `resolution`
/// triangles.
fn generate_circle_vertices(radius: f32, resolution: usize) -> Vec<PosTex> {
    use std::f32::consts::PI;

    let mut vertices = Vec::with_capacity(resolution * 3);
    let angle_offset = 2.0 * PI / resolution as f32;

    // Helper function to generate the vertex at the specified angle.
    let generate_vertex = |angle: f32| {
        let x = angle.cos();
        let y = angle.sin();
        PosTex {
            position: [x * radius, y * radius, 0.0],
            tex_coord: [x, y],
        }
    };

    for index in 0..resolution {
        vertices.push(PosTex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        });

        vertices.push(generate_vertex(angle_offset * index as f32));
        vertices.push(generate_vertex(angle_offset * (index + 1) as f32));
    }

    vertices
}

/// Generates six vertices forming a rectangle.
fn generate_rectangle_vertices(left: f32, bottom: f32, right: f32, top: f32) -> Vec<PosTex> {
    vec![
        PosTex {
            position: [left, bottom, 0.],
            tex_coord: [0.0, 0.0],
        },
        PosTex {
            position: [right, bottom, 0.0],
            tex_coord: [1.0, 0.0],
        },
        PosTex {
            position: [left, top, 0.0],
            tex_coord: [1.0, 1.0],
        },
        PosTex {
            position: [right, top, 0.],
            tex_coord: [1.0, 1.0],
        },
        PosTex {
            position: [left, top, 0.],
            tex_coord: [0.0, 1.0],
        },
        PosTex {
            position: [right, bottom, 0.0],
            tex_coord: [0.0, 0.0],
        },
    ]
}
