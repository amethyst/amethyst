//! TODO: Rewrite for new renderer.

extern crate amethyst;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::config::Config;
use amethyst::ecs::{Component, Fetch, FetchMut, Join, System, VecStorage, World, WriteStorage};
use amethyst::ecs::components::{Mesh, LocalTransform, Texture, Transform};
use amethyst::ecs::resources::{Camera, InputHandler, Projection, Time};
use amethyst::ecs::systems::TransformSystem;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Pipeline, VertexPosNormal};

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
        let (left_bound, right_bound, top_bound, bottom_bound) = match camera.proj {
            Projection::Orthographic {
                left,
                right,
                top,
                bottom,
                ..
            } => (left, right, top, bottom),
            _ => (1.0, 1.0, 1.0, 1.0),
        };

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
                    // If `W` is pressed and plank is in screen boundaries then move up
                    if input.key_is_pressed(VirtualKeyCode::W) {
                        if plank.position + plank.dimensions[1] / 2. < 1. {
                            plank.position += plank.velocity * delta_time;
                        }
                    }
                    // If `S` is pressed and plank is in screen boundaries then move down
                    if input.key_is_pressed(VirtualKeyCode::S) {
                        if plank.position - plank.dimensions[1] / 2. > -1. {
                            plank.position -= plank.velocity * delta_time;
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
                    // If `Up` is pressed and plank is in screen boundaries then move down
                    if input.key_is_pressed(VirtualKeyCode::Up) {
                        if plank.position + plank.dimensions[1] / 2. < top_bound {
                            plank.position += plank.velocity * delta_time;
                        }
                    }
                    // If `Down` is pressed and plank is in screen boundaries then move down
                    if input.key_is_pressed(VirtualKeyCode::Down) {
                        if plank.position - plank.dimensions[1] / 2. > bottom_bound {
                            plank.position -= plank.velocity * delta_time;
                        }
                    }
                    // Set translation[0] of renderable corresponding to this plank
                    local.translation[0] = right_bound - plank.dimensions[0] / 2.0
                }
            };
            // Set translation[1] of renderable corresponding to this plank
            local.translation[1] = plank.position;
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
    fn on_start(&mut self, world: &mut World, assets: &mut AssetManager, pipe: &mut Pipeline) {
        use amethyst::ecs::resources::{Camera, InputHandler, Projection, ScreenDimensions};
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::{Clear, DrawFlat};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawFlat::new("main", "main")]);

        pipe.layers.push(layer);

        {
            let dim = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let aspect_ratio = dim.aspect_ratio;
            let eye = [0., 0., 0.1];
            let target = [0., 0., 0.];
            let up = [0., 1., 0.];

            // Get an Orthographic projection
            let proj = Projection::Orthographic {
                left: -1.0 * aspect_ratio,
                right: 1.0 * aspect_ratio,
                bottom: -1.0,
                top: 1.0,
                near: 0.0,
                far: 1.0,
            };

            camera.proj = proj;
            camera.eye = eye;
            camera.target = target;
            camera.up = up;
        }

        // Add all resources
        world.add_resource::<Score>(Score::new());
        world.add_resource::<InputHandler>(InputHandler::new());

        // Generate a square mesh
        assets.register_asset::<Mesh>();
        assets.register_asset::<Texture>();
        assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        let square_verts = gen_rectangle(1.0, 1.0);
        assets.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("square", square_verts);
        let square = assets
            .create_renderable("square", "white", "white", "white", 1.0)
            .unwrap();

        // Create a ball entity
        let mut ball = Ball::new();
        ball.size = 0.02;
        ball.velocity = [0.5, 0.5];
        world
            .create_entity()
            .with(square.clone())
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
            .with(square.clone())
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
            .with(square.clone())
            .with(plank)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     world: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
                     -> Trans {
        use amethyst::ecs::resources::InputHandler;

        let mut input = world.write_resource::<InputHandler>();
        input.update(events);

        for e in events {
            match **e {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/04_pong/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let mut game = Application::build(Pong, cfg)
        .register::<Ball>()
        .register::<Plank>()
        .with::<PongSystem>(PongSystem, "pong_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &["pong_system"])
        .done();
    game.run();
}

fn gen_rectangle(w: f32, h: f32) -> Vec<VertexPosNormal> {
    let data: Vec<VertexPosNormal> = vec![VertexPosNormal {
                                              pos: [-w / 2., -h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [0., 0.],
                                          },
                                          VertexPosNormal {
                                              pos: [w / 2., -h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [1., 0.],
                                          },
                                          VertexPosNormal {
                                              pos: [w / 2., h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [1., 1.],
                                          },
                                          VertexPosNormal {
                                              pos: [w / 2., h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [1., 1.],
                                          },
                                          VertexPosNormal {
                                              pos: [-w / 2., h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [1., 1.],
                                          },
                                          VertexPosNormal {
                                              pos: [-w / 2., -h / 2., 0.],
                                              normal: [0., 0., 1.],
                                              tex_coord: [1., 1.],
                                          }];
    data
}
