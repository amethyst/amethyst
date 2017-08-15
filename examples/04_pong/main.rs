//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate amethyst_renderer;
extern crate cgmath;

//use amethyst::{Application, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::prelude::*;
use amethyst::timing::Time;
use amethyst::ecs::{Component, Fetch, FetchMut, Join, System, VecStorage, World, WriteStorage};
use amethyst::ecs::components::*;
use amethyst::ecs::resources::input::*;
use amethyst::ecs::systems::{RenderSystem, TransformSystem};
use amethyst_renderer::prelude::*;
use cgmath::{Matrix4, Deg, Vector3};


struct Pong;

struct Ball {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub size: f32,
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            position: [0.0, 0.0],
            velocity: [-1.0, -1.0],
            size: 1.0,
        }
    }
}

impl Component for Ball {
    type Storage = VecStorage<Ball>;
}

enum Side {
    Left,
    Right,
}

struct Plank {
    pub position: f32,
    pub velocity: f32,
    pub dimensions: [f32; 2],
    pub side: Side,
}

impl Plank {
    pub fn new(side: Side) -> Plank {
        Plank {
            position: 0.,
            velocity: 1.,
            dimensions: [1., 1.],
            side: side,
        }
    }
}

impl Component for Plank {
    type Storage = VecStorage<Plank>;
}

struct PongSystem;

struct Score {
    score_left: i32,
    score_right: i32,
}

impl Score {
    pub fn new() -> Score {
        Score {
            score_left: 0,
            score_right: 0,
        }
    }
}

// Pong game system
impl<'a> System<'a> for PongSystem {
    type SystemData = (WriteStorage<'a, Ball>,
     WriteStorage<'a, Plank>,
     WriteStorage<'a, LocalTransform>,
     Fetch<'a, Camera>,
     Fetch<'a, Time>,
     Fetch<'a, InputHandler>,
     FetchMut<'a, Score>);

    fn run(&mut self,
           (mut balls, mut planks, mut locals, camera, time, input, mut score): Self::SystemData) {
        // Get left and right boundaries of the screen
        let (left_bound, right_bound, top_bound, bottom_bound) = (0.0, 1.0, 1.0, 0.0);

        // Properties of left paddle.
        let mut left_dimensions = [0.0, 0.0];
        let mut left_position = 0.0;

        // Properties of right paddle.
        let mut right_dimensions = [0.0, 0.0];
        let mut right_position = 0.0;

        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        // Process all planks
        for (plank, local) in (&mut planks, &mut locals).join() {
            match plank.side {
                // If it is a left plank
                Side::Left => {
                    // Store left plank position for later use in ball processing
                    left_position = plank.position;
                    // Store left plank dimensions for later use in ball processing
                    left_dimensions = plank.dimensions;
                    // Move plank according to axis input.
                    if let Some(value) = input.axis_value("P1") {
                            plank.position += plank.velocity * delta_time * value as f32;
                        if plank.position + plank.dimensions[1] / 2. > 1. {
                            plank.position = 1. - plank.dimensions[1] / 2.
                        }
                        if plank.position - plank.dimensions[1] / 2. < 0. {
                            plank.position = plank.dimensions[1] / 2.;
                        }
                    }
                    // Set translation[0] of renderable corresponding to this plank
                    local.translation[0] = left_bound + plank.dimensions[0] / 2.0
                }
                // If it is a right plank
                Side::Right => {
                    // Store right plank position for later use in ball processing
                    right_position = plank.position;
                    // Store right plank dimensions for later use in ball processing
                    right_dimensions = plank.dimensions;
                    // Move plank according to axis input.
                    if let Some(value) = input.axis_value("P2") {
                        plank.position += plank.velocity * delta_time * value as f32;
                        if plank.position + plank.dimensions[1] / 2. > 1. {
                            plank.position = 1. - plank.dimensions[1] / 2.
                        }
                        if plank.position - plank.dimensions[1] / 2. < 0. {
                            plank.position = plank.dimensions[1] / 2.;
                        }
                    }
                    // Set translation[0] of renderable corresponding to this plank
                    local.translation[0] = right_bound - plank.dimensions[0] / 2.0
                }
            };
            // Set translation[1] of renderable corresponding to this plank
            local.translation[1] = plank.position * (top_bound - bottom_bound) + bottom_bound;
            // Set scale for renderable corresponding to this plank
            local.scale = [plank.dimensions[0], plank.dimensions[1], 1.0];
        }

        // Process the ball
        for (ball, local) in (&mut balls, &mut locals).join() {
            // Move the ball
            ball.position[0] += ball.velocity[0] * delta_time;
            ball.position[1] += ball.velocity[1] * delta_time;

            // Check if the ball has collided with the right plank
            if ball.position[0] + ball.size / 2. > right_bound - left_dimensions[0] &&
               ball.position[0] + ball.size / 2. < right_bound {
                if ball.position[1] - ball.size / 2. < right_position + right_dimensions[1] / 2. &&
                   ball.position[1] + ball.size / 2. > right_position - right_dimensions[1] / 2. {
                    ball.position[0] = right_bound - right_dimensions[0] - ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                }
            }

            // Check if the ball is to the left of the right boundary
            // if it is not reset it's position and score the left player
            if ball.position[0] - ball.size / 2. > right_bound {
                ball.position[0] = 0.;
                score.score_left += 1;
                println!("Left player score: {0}, Right player score {1}",
                         score.score_left,
                         score.score_right);
            }

            // Check if the ball has collided with the left plank
            if ball.position[0] - ball.size / 2. < left_bound + left_dimensions[0] &&
               ball.position[0] + ball.size / 2. > left_bound {
                if ball.position[1] - ball.size / 2. < left_position + left_dimensions[1] / 2. &&
                   ball.position[1] + ball.size / 2. > left_position - left_dimensions[1] / 2. {
                    ball.position[0] = left_bound + left_dimensions[0] + ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                }
            }

            // Check if the ball is to the right of the left boundary
            // if it is not reset it's position and score the right player
            if ball.position[0] + ball.size / 2. < left_bound {
                ball.position[0] = 0.;
                score.score_right += 1;
                println!("Left player score: {0}, Right player score {1}",
                         score.score_left,
                         score.score_right);
            }

            // Check if the ball is below the top boundary, if it is not deflect it
            if ball.position[1] + ball.size / 2. > top_bound {
                ball.position[1] = top_bound - ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
            }

            // Check if the ball is above the bottom boundary, if it is not deflect it
            if ball.position[1] - ball.size / 2. < bottom_bound {
                ball.position[1] = bottom_bound + ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
            }

            // Update the renderable corresponding to this ball
            local.translation[0] = ball.position[0];
            local.translation[1] = ball.position[1];
            local.scale[0] = ball.size;
            local.scale[1] = ball.size;
        }
    }
}

impl State for Pong {
    fn on_start(&mut self, engine: &mut Engine) {
        let world = &mut engine.world;
        
        world.add_resource(Camera {
            eye: [0., 0., 1.0].into(),
            proj: Projection::orthographic(0.0, 1.0, 1.0, 0.0).into(),
            forward: [0., 0., -1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0., 1.0, 0.].into(),
        });

        // Add all resources
        world.add_resource(Score::new());
        let mut input = InputHandler::new();
        input.bindings = Bindings::load(format!("{}/examples/04_pong/resources/input.ron",
                                                env!("CARGO_MANIFEST_DIR")));
        world.add_resource(input);
        world.add_resource(Time::default());

        world.register::<Transform>();
        world.register::<Child>();
        world.register::<Init>();
        world.register::<LocalTransform>();
        world.register::<MeshComponent>();
        world.register::<MaterialComponent>();
        world.register::<LightComponent>();
        world.register::<Unfinished<MeshComponent>>();
        world.register::<Unfinished<MaterialComponent>>();

        // Generate a square mesh
        let tex = Texture::from_color_val([1.0, 1.0, 1.0, 1.0]);
        let mtl = MaterialBuilder::new().with_albedo(tex);

        let square_verts = gen_rectangle(1.0, 1.0);
        let mesh = Mesh::build(square_verts);

        // Create a ball entity
        let mut ball = Ball::new();
        ball.size = 0.02;
        ball.velocity = [0.5, 0.5];
        world
            .create_entity()
            .with(mesh.clone().unfinished())
            .with(mtl.clone().unfinished())
            .with(ball)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        // Create a left plank entity
        let mut plank = Plank::new(Side::Left);
        plank.dimensions[0] = 0.01;
        plank.dimensions[1] = 0.1;
        plank.velocity = 1.;
        world
            .create_entity()
            .with(mesh.clone().unfinished())
            .with(mtl.clone().unfinished())
            .with(plank)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        // Create right plank entity
        let mut plank = Plank::new(Side::Right);
        plank.dimensions[0] = 0.01;
        plank.dimensions[1] = 0.1;
        plank.velocity = 1.;
        world
            .create_entity()
            .with(mesh.unfinished())
            .with(mtl.unfinished())
            .with(plank)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }

    fn handle_event(&mut self, engine: &mut Engine, event: Event) -> Trans {
        let mut input = engine.world.write_resource::<InputHandler>();        
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape), ..
                    }, ..
                } | WindowEvent::Closed => {
                    Trans::Quit
                }
                _ => {
                    input.update(&[event]);
                    Trans::None
                }
            },
            _ => Trans::None,
        }
    }
}

fn main() {
    let path = format!("{}/examples/04_pong/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));

    let builder = Application::build(Pong);

    let render = RenderSystem::new(
        &builder.events,
        DisplayConfig::default(),
        Pipeline::build()
            .with_stage(Stage::with_backbuffer()
                .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                .with_model_pass(pass::DrawFlat::<PosNormTex>::new())
            )
    ).unwrap();

    let mut game = builder
        .register::<Ball>()
        .register::<Plank>()
        .with::<PongSystem>(PongSystem, "pong_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &["pong_system"])
        .with_thread_local(render)
        .build()
        .expect("Fatal error");
    game.run();
}

fn gen_rectangle(w: f32, h: f32) -> Vec<PosNormTex> {
    let data: Vec<PosNormTex> = vec![PosNormTex {
                                              a_position: [-w / 2., -h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [0., 0.],
                                          },
                                          PosNormTex {
                                              a_position: [w / 2., -h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [1., 0.],
                                          },
                                          PosNormTex {
                                              a_position: [w / 2., h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [1., 1.],
                                          },
                                          PosNormTex {
                                              a_position: [w / 2., h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [1., 1.],
                                          },
                                          PosNormTex {
                                              a_position: [-w / 2., h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [1., 1.],
                                          },
                                          PosNormTex {
                                              a_position: [-w / 2., -h / 2., 0.],
                                              a_normal: [0., 0., 1.],
                                              a_tex_coord: [1., 1.],
                                          }];
    data
}
