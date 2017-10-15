use {ARENA_HEIGHT, ARENA_WIDTH};
use {Ball, Paddle, Side};
use amethyst::assets::Loader;
use amethyst::ecs::World;
use amethyst::ecs::transform::{LocalTransform, Transform};
use amethyst::prelude::*;
use amethyst::renderer::MeshHandle;
use amethyst::renderer::prelude::*;

pub struct Pong;

impl State for Pong {
    fn on_start(&mut self, engine: &mut Engine) {
        use audio::initialise_audio;

        // Setup our game.
        initialise_paddles(&mut engine.world);
        initialise_balls(&mut engine.world);
        initialise_camera(&mut engine.world);
        initialise_audio(&mut engine.world);
        hide_cursor(&mut engine.world);
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    world.add_resource(Camera {
        eye: [0.0, 0.0, 1.0].into(),
        // Make the arena fit perfectly in the view of the camera.
        proj: Projection::orthographic(0.0, ARENA_WIDTH, ARENA_HEIGHT, 0.0).into(),
        forward: [0., 0., -1.0].into(),
        right: [1.0, 0.0, 0.0].into(),
        up: [0., 1.0, 0.].into(),
    });
}

/// Hide the cursor so it's invisible while playing.
fn hide_cursor(world: &mut World) {
    use amethyst::ecs::rendering::resources::WindowMessages;
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
    use {PADDLE_COLOUR, PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

    let mut left_transform = LocalTransform::default();
    let mut right_transform = LocalTransform::default();

    // Correctly position the paddles.
    let y = (ARENA_HEIGHT - PADDLE_HEIGHT) / 2.0;
    left_transform.translation = [0.0, y, 0.0];
    right_transform.translation = [ARENA_WIDTH - PADDLE_WIDTH, y, 0.0];

    // Create the mesh and the material needed.
    let mesh = create_mesh(
        world,
        generate_rectangle_vertices(0.0, 0.0, PADDLE_WIDTH, PADDLE_HEIGHT),
    );

    let material = create_colour_material(world, PADDLE_COLOUR);

    // Create a left plank entity.
    world
        .create_entity()
        .with(mesh.clone())
        .with(material.clone())
        .with(Paddle {
            side: Side::Left,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        })
        .with(Transform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(Paddle {
            side: Side::Right,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
        })
        .with(Transform::default())
        .with(right_transform)
        .build();
}

/// Initialises one ball in the middle-ish of the arena.
fn initialise_balls(world: &mut World) {
    use {ARENA_HEIGHT, ARENA_WIDTH, BALL_COLOUR, BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y};

    // Create the mesh, material and translation.
    let mesh = create_mesh(world, generate_circle_vertices(BALL_RADIUS, 16));
    let material = create_colour_material(world, BALL_COLOUR);
    let mut local_transform = LocalTransform::default();
    local_transform.translation = [ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0];

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .with(Transform::default())
        .build();
}

/// Converts a vector of vertices into a mesh.
fn create_mesh(world: &World, vertices: Vec<PosTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), &world.read_resource())
}

/// Creates a solid material of the specified colour.
fn create_colour_material(world: &World, colour: [f32; 4]) -> Material {
    // TODO: optimize

    use amethyst::renderer::MaterialDefaults;

    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let albedo = loader.load_from_data(colour.into(), &world.read_resource());

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
