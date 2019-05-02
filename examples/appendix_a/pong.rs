use crate::{
    config::{ArenaConfig, BallConfig, PaddlesConfig},
    systems::ScoreText,
    Ball, Paddle, Side,
};
use amethyst::{
    assets::Loader,
    core::{
        math::{Vector2, Vector3},
        Transform,
    },
    ecs::prelude::World,
    prelude::*,
    renderer::{
        Camera, Material, MaterialDefaults, MeshHandle, PosTex, Projection, WindowMessages,
    },
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

pub struct Pong;

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        use crate::audio::initialise_audio;

        // Setup our game.
        initialise_paddles(world);
        initialise_balls(world);
        initialise_camera(world);
        initialise_audio(world);
        initialise_score(world);
        hide_cursor(world);
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let (arena_height, arena_width) = {
        let config = &world.read_resource::<ArenaConfig>();
        (config.height, config.width)
    };

    let mut transform = Transform::<f32>::default();
    transform.set_translation_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            arena_width,
            0.0,
            arena_height,
        )))
        .with(transform)
        .build();
}

/// Hide the cursor, so it's invisible while playing.
fn hide_cursor(world: &mut World) {
    world
        .write_resource::<WindowMessages>()
        .send_command(|win| win.hide_cursor(true));
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World) {
    let mut left_transform = Transform::<f32>::default();
    let mut right_transform = Transform::<f32>::default();

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
    left_transform.set_translation_xyz(0.0, left_y, 0.0);
    right_transform.set_translation_xyz(arena_width - right_width, right_y, 0.0);

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
    let mut local_transform = Transform::<f32>::default();
    local_transform.set_translation_xyz(arena_width / 2.0, arena_height / 2.0, 0.0);

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(Ball {
            radius: radius,
            velocity: [velocity_x, velocity_y],
        })
        .with(local_transform)
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
    let p1_transform =
        UiTransform::new("P1".to_string(), Anchor::TopMiddle, -50., 50., 1., 55., 50.);

    let p2_transform =
        UiTransform::new("P2".to_string(), Anchor::TopMiddle, 50., 50., 1., 55., 50.);

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
            position: Vector3::new(x * radius, y * radius, 0.0),
            tex_coord: Vector2::new(x, y),
        }
    };

    for index in 0..resolution {
        vertices.push(PosTex {
            position: Vector3::new(0.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
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
            position: Vector3::new(left, bottom, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
        },
        PosTex {
            position: Vector3::new(right, bottom, 0.0),
            tex_coord: Vector2::new(1.0, 0.0),
        },
        PosTex {
            position: Vector3::new(left, top, 0.0),
            tex_coord: Vector2::new(1.0, 1.0),
        },
        PosTex {
            position: Vector3::new(right, top, 0.0),
            tex_coord: Vector2::new(1.0, 1.0),
        },
        PosTex {
            position: Vector3::new(left, top, 0.0),
            tex_coord: Vector2::new(0.0, 1.0),
        },
        PosTex {
            position: Vector3::new(right, bottom, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
        },
    ]
}
