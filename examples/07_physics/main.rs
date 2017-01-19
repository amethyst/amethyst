//! Demonstrates nphysics integration with a simple example
//! of various shapes falling onto a mesh.

extern crate amethyst;
extern crate cgmath;
extern crate genmesh;
extern crate gfx;
extern crate nalgebra as na;
extern crate rand;

use std::cmp::max;
use std::sync::Arc;


use amethyst::asset_manager::AssetManager;
use amethyst::components::physics::{PhysicsComponent, RigidBody, Ball, Cuboid, TriMesh3};
use amethyst::components::rendering::{Mesh, Texture, TextureLoadData};
use amethyst::components::transform::{LocalTransform, Transform};
use amethyst::config::Element;
use amethyst::ecs::{Join, Processor, RunArg, World};
use amethyst::engine::{Application, State, Trans};
use amethyst::event::{WindowEvent, Event, VirtualKeyCode};
use amethyst::gfx_device::DisplayConfig;
use amethyst::processors::physics::PhysicsWorld;
use amethyst::renderer::{Layer, Light, Pipeline, VertexPosNormal};
use amethyst::renderer::pass::{Clear, DrawShaded};
use amethyst::world_resources::camera::{Camera, Projection};
use amethyst::world_resources::{InputHandler, ScreenDimensions, Time};
use gfx::tex::{AaMode, Kind};

use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::genmesh::generators::{Cube, SphereUV};
use self::cgmath::{InnerSpace, Vector3};


// Generates a curved terrain mesh with some noise
fn gen_terrain(width: usize, height: usize) -> Vec<VertexPosNormal> {
    let mut mat: na::DMatrix<f32> = na::DMatrix::new_zeros(width + 1, height + 1);

    // Generate values in the matrix representing a curve
    for i in 0..width + 1 {
        for j in 0..height + 1 {
            // Calculate -(i^2 + j^2), move to the center, flatten a bit,
            // and finally add some noise.
            let i_2 = ((i as i32 - width as i32 / 2) as f32).powf(2.0);
            let j_2 = ((j as i32 - height as i32 / 2) as f32).powf(2.0);
            mat[(i, j)] = -(i_2 + j_2) / 128.0 - (rand::random::<f32>() / 2.0);
        }
    }

    // Turn matrix into a mesh consisting of a `VertexPosNormal`s.
    let mut vertices = vec![];

    for i in 0..width {
        for j in 0..height {
            // Convert to float, and center around (0, 0)
            let ii = (i as i32 - width as i32 / 2) as f32;
            let jj = (j as i32 - height as i32 / 2) as f32;

            // Calculate vertices for each mesh face and convert to `VertexPosNormal`
            if i % 2 == 0 {
                let verts = vec![
                    Vector3::new(ii      , jj      , mat[(i            , j    )]),
                    Vector3::new(ii + 1.0, jj      , mat[(i + 1        , j    )]),
                    Vector3::new(ii      , jj + 1.0, mat[(i            , j + 1)]),
                    Vector3::new(ii - 1.0, jj + 1.0, mat[(max(i, 1) - 1, j + 1)]),
                ];

                let u = verts[1] - verts[0];
                let v = verts[2] - verts[0];
                let w = verts[3] - verts[0];

                let normals = [
                    u.cross(v),
                    v.cross(w),
                ];

                vertices.extend(vec![
                    VertexPosNormal {
                        pos: verts[0].into(),
                        normal: normals[0].into(),
                        tex_coord: [0.0, 0.0],
                    },
                    VertexPosNormal {
                        pos: verts[1].into(),
                        normal: normals[0].into(),
                        tex_coord: [1.0, 0.0],
                    },
                    VertexPosNormal {
                        pos: verts[2].into(),
                        normal: normals[0].into(),
                        tex_coord: [0.0, 1.0],
                    },
                ]);

                if 0 < i {
                    vertices.extend(vec![
                        VertexPosNormal {
                            pos: verts[0].into(),
                            normal: normals[1].into(),
                            tex_coord: [1.0, 0.0],
                        },
                        VertexPosNormal {
                            pos: verts[2].into(),
                            normal: normals[1].into(),
                            tex_coord: [1.0, 1.0],
                        },
                        VertexPosNormal {
                            pos: verts[3].into(),
                            normal: normals[1].into(),
                            tex_coord: [0.0, 1.0],
                        }
                    ]);
                }
            } else {
                let verts = vec![
                    Vector3::new(ii      , jj      , mat[(i    , j    )]),
                    Vector3::new(ii      , jj + 1.0, mat[(i    , j + 1)]),
                    Vector3::new(ii - 1.0, jj + 1.0, mat[(i - 1, j + 1)]),
                    Vector3::new(ii + 1.0, jj      , mat[(i + 1, j    )]),
                ];

                let u = verts[1] - verts[0];
                let v = verts[2] - verts[0];
                let w = verts[3] - verts[0];

                let normals = [
                    u.cross(v),
                    w.cross(v),
                ];

                vertices.extend(vec![
                    VertexPosNormal {
                        pos: verts[0].into(),
                        normal: normals[0].into(),
                        tex_coord: [1.0, 0.0],
                    },
                    VertexPosNormal {
                        pos: verts[1].into(),
                        normal: normals[0].into(),
                        tex_coord: [1.0, 1.0],
                    },
                    VertexPosNormal {
                        pos: verts[2].into(),
                        normal: normals[0].into(),
                        tex_coord: [0.0, 1.0],
                    },
                ]);

                if i < width - 1 {
                    vertices.extend(vec![
                        VertexPosNormal {
                            pos: verts[0].into(),
                            normal: normals[1].into(),
                            tex_coord: [0.0, 0.0],
                        },
                        VertexPosNormal {
                            pos: verts[1].into(),
                            normal: normals[1].into(),
                            tex_coord: [0.0, 1.0],
                        },
                        VertexPosNormal {
                            pos: verts[3].into(),
                            normal: normals[1].into(),
                            tex_coord: [1.0, 0.0],
                        }
                    ]);
                }
            }
        }
    }

    vertices
}

pub fn gen_cube() -> Vec<VertexPosNormal> {
    Cube::new()
        .vertex(|(x, y, z)| {
            VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.],
            }
        })
        .triangulate()
        .vertices()
        .collect::<Vec<VertexPosNormal>>()
}

fn gen_sphere(u: usize, v: usize) -> Vec<VertexPosNormal> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.],
            }
        })
        .triangulate()
        .vertices()
        .collect::<Vec<VertexPosNormal>>()
}

/// Struct for running physics example
struct PhysicsDemo;

/// Struct for storing camera angle to make it available inside a `Processor`
struct CameraAngle {
    angle: f32,
}

// Set up Processor for demo
struct PhysicsDemoProcessor;

impl Processor<()> for PhysicsDemoProcessor {
    fn run(&mut self, arg: RunArg, _: ()) {
        let (
            mut bodies,
            mut camera,
            mut angle,
            time,
        ) = arg.fetch(|w| (
            w.write::<PhysicsComponent>(),
            w.write_resource::<Camera>(),
            w.write_resource::<CameraAngle>(),
            w.read_resource::<Time>(),
        ));

        // Rotate camera around origin
        angle.angle -= time.delta_time.subsec_nanos() as f32 / 1.0e10;

        camera.eye[0] = 25.0 * angle.angle.cos();
        camera.eye[1] = 25.0 * angle.angle.sin();

        // Reset bodies when they fall too far
        for body in (&mut bodies).iter() {
            let mut body = body.borrow_mut();
            if body.position().translation[2] < -40.0 {
                body.set_translation(na::Vector3::new(
                     5.0 * rand::random::<f32>(),
                     5.0 * rand::random::<f32>(),
                    40.0 * rand::random::<f32>() + 30.0,
                ));

                body.set_rotation(na::Vector3::new(0.0, 0.0, 0.0));
                body.set_lin_vel(na::Vector3::new(0.0, 0.0, 0.0));
            }
        }

    }
}

impl State for PhysicsDemo {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipeline: &mut Pipeline) {
        {
            let dimensions = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let proj = Projection::Perspective {
                fov: 60.0,
                aspect_ratio: dimensions.aspect_ratio,
                near: 1.0,
                far: 100.0,
            };
            camera.projection = proj;
            camera.eye =    [0.0, -30.0, 10.0];
            camera.target = [0.0,   0.0,  0.0];
            camera.up =     [0.0,   0.0,  1.0];

            // Instantiate physics world
            let mut physics_world = world.write_resource::<PhysicsWorld>();
            physics_world.set_gravity(na::Vector3::new(0.0, 0.0, -9.81));
        }

        // Set up rendering pipeline
        let layer = Layer::new("main", vec![
            Clear::new([0.0, 0.0, 0.0, 1.0]),
            DrawShaded::new("main", "main"),
        ]);
        pipeline.layers = vec![layer];

        world.add_resource::<CameraAngle>(CameraAngle { angle: 0.0 });
        world.add_resource::<InputHandler>(InputHandler::new());

        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 10.0,
            center: [5.0, 5.0, 10.0],
            propagation_constant: 0.0,
            propagation_linear: 0.0,
            propagation_r_square: 50.0,
        };

        world.create_now()
            .with(light)
            .build();

        // Initialize asset manager
        asset_manager.register_asset::<Mesh>();
        asset_manager.register_asset::<Texture>();

        // Create some colors for the world.
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("crate", [0.4, 0.2, 0.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("tan",   [0.7, 0.5, 0.3, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("black", [0.0, 0.0, 0.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("gray",  [0.3, 0.3, 0.3, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.7, 1.0, 0.7, 1.0]);

        // Create a manual greenish texture
        let data = TextureLoadData {
            kind: Kind::D2(4, 2, AaMode::Single),
            raw: &[
                &[[0,  92,  9, 255], [0, 104, 10, 255], [0, 123, 12, 255], [1, 142, 14, 255]],
                &[[0, 104, 10, 255], [0, 123, 12, 255], [1, 142, 14, 255], [1, 166, 17, 255]],
            ],
        };
        asset_manager.load_asset_from_data::<Texture, TextureLoadData>("grassy", data);

        // Generate the renderable meshes
        let cube_vertices = gen_cube();
        let sphere_vertices = gen_sphere(32, 32);
        let data = gen_terrain(30, 30);

        asset_manager.load_asset_from_data::<Mesh, Vec<_>>("cube", cube_vertices);
        asset_manager.load_asset_from_data::<Mesh, Vec<_>>("sphere", sphere_vertices);
        asset_manager.load_asset_from_data::<Mesh, Vec<_>>("terrain", data.clone());

        let cube = asset_manager.create_renderable("cube", "crate", "tan").unwrap();
        let sphere = asset_manager.create_renderable("sphere", "black", "gray").unwrap();
        let terrain = asset_manager.create_renderable("terrain", "grassy", "green").unwrap();

        // Add terrain to world
        let vertices = data
            .iter()
            .map(|v| na::Point3::new(v.pos[0], v.pos[1], v.pos[2]))
            .collect();

        let indices = (0..data.iter().len() / 3).map(|i| {
            let j = 3 * i;
            na::Point3::new(j, j + 1, j + 2)
        }).collect();

        let mesh: TriMesh3<f32> = TriMesh3::new(Arc::new(vertices), Arc::new(indices), None, None);
        let body = RigidBody::new_static(mesh, 0.3, 0.6);
        let handle = {
            let mut physics_world = world.write_resource::<PhysicsWorld>();
            physics_world.add_rigid_body(body)
        };
        world.create_now()
            .with(terrain.clone())
            .with(PhysicsComponent(handle))
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        // Generate some cube and ball entities
        for i in 0..100 {
            let translation = na::Vector3::new(
                5.0 * rand::random::<f32>(),
                5.0 * rand::random::<f32>(),
                5.0 * i as f32 + 30.0,
            );

            if i % 2 == 0 {
                let shape = Cuboid::new(na::Vector3::new(1.0, 1.0, 1.0));
                let mut body = RigidBody::new_dynamic(shape, 1.0, 0.3, 0.6);
                body.append_translation(&translation);
                let handle = {
                    let mut physics_world = world.write_resource::<PhysicsWorld>();
                    physics_world.add_rigid_body(body)
                };
                let mut transform = LocalTransform::default();
                transform.translation = *translation.as_ref();
                world.create_now()
                    .with(cube.clone())
                    .with(PhysicsComponent(handle))
                    .with(transform)
                    .with(Transform::default())
                    .build();
            } else {
                let shape = Ball::new(1.0);
                let mut body = RigidBody::new_dynamic(shape, 100.0, 0.3, 0.6);
                body.append_translation(&translation);
                let handle = {
                    let mut physics_world = world.write_resource::<PhysicsWorld>();
                    physics_world.add_rigid_body(body)
                };
                let mut transform = LocalTransform::default();
                transform.translation = *translation.as_ref();
                world.create_now()
                    .with(sphere.clone())
                    .with(PhysicsComponent(handle))
                    .with(transform)
                    .with(Transform::default())
                    .build();
            }
        }
    }

    // Quit on escape or window close
    fn handle_events(&mut self, events: &[WindowEvent], world: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        let mut input_handler = world.write_resource::<InputHandler>();
        input_handler.update(events);
        for event in events {
            match event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/07_physics/resources/config.yml",
                        env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::from_file(path).unwrap();

    let mut game = Application::build(PhysicsDemo, display_config)
                   .with::<PhysicsDemoProcessor>(PhysicsDemoProcessor, "physics_demo_processor", 2)
                   .done();
    game.run();
}
