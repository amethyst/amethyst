extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::processors::rendering::{RenderingProcessor, Renderable, Light, Camera, Projection};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Join, VecStorage, Component, Processor, RunArg};
use std::sync::{Mutex, Arc};

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

struct PongProcessor;

unsafe impl Sync for PongProcessor {  }

// If right_up == true then the right plank will move up,
// other fields work the same way
struct InputState {
    pub right_up: bool,
    pub right_down: bool,
    pub left_up: bool,
    pub left_down: bool,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            right_up: false,
            right_down: false,
            left_up: false,
            left_down: false,
        }
    }
}

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

// Pong game processor
impl Processor<Arc<Mutex<Context>>> for PongProcessor {
    fn run(&mut self, arg: RunArg, context: Arc<Mutex<Context>>) {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode, ElementState};
        use std::ops::Deref;

        // Get all needed component storages and resources
        let context = context.lock().unwrap();
        let (mut balls,
             mut planks,
             mut renderables,

             mut input_state,
             projection,
             mut score) = arg.fetch(|w| (w.write::<Ball>(),
                                         w.write::<Plank>(),
                                         w.write::<Renderable>(),

                                         w.write_resource::<InputState>(),
                                         w.read_resource::<Projection>(),
                                         w.write_resource::<Score>()));

        // Get left and right boundaries of the screen
        let (left_boundary, right_boundary, top_boundary, bottom_boundary) = match *projection.deref() {
            Projection::Orthographic {
                left,
                right,
                top,
                bottom,
                ..
            } => (left, right, top, bottom),
            _ => (1.0, 1.0, 1.0, 1.0),
        };

        // Update input_state resource using incoming events
        let engine_events = context.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => input_state.right_up = true,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => input_state.right_down = true,
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Up)) => input_state.right_up = false,
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Down)) => input_state.right_down = false,

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => input_state.left_up = true,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => input_state.left_down = true,
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::W)) => input_state.left_up = false,
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::S)) => input_state.left_down = false,

                _ => (),
            }
        }

        // Position of left plank
        let mut left_position = 0.;
        // Position of right plank
        let mut right_position = 0.;

        // Dimensions of left plank
        let mut left_dimensions = [0., 0.];
        // Dimensions of right plank
        let mut right_dimensions = [0., 0.];

        // Process all planks
        for (plank, renderable) in (&mut planks, &mut renderables).iter() {
            match plank.side {
                // If it is a left plank
                Side::Left => {
                    // Store left plank position for later use in ball processing
                    left_position = plank.position;
                    // Store left plank dimensions for later use in ball processing
                    left_dimensions = plank.dimensions;
                    // If input_state.left_up == true and plank is in screen boundaries then move up
                    if input_state.left_up {
                        if plank.position + plank.dimensions[1]/2. < 1. {
                            plank.position += plank.velocity * context.delta_time.num_milliseconds() as f32 / 1000.0;
                        }
                    }
                    // If input_state.left_down == true and plank is in screen boundaries then move down
                    if input_state.left_down {
                        if plank.position - plank.dimensions[1]/2. > -1. {
                            plank.position -= plank.velocity * context.delta_time.num_milliseconds() as f32 / 1000.0;
                        }
                    }
                    // Set translation[0] of renderable corresponding to this plank
                    renderable.translation[0] = left_boundary + plank.dimensions[0]/2.;
                }
                // If it is a right plank
                Side::Right => {
                    // Store right plank position for later use in ball processing
                    right_position = plank.position;
                    // Store right plank dimensions for later use in ball processing
                    right_dimensions = plank.dimensions;
                    // If input_state.right_up == true and plank is in screen boundaries then move down
                    if input_state.right_up {
                        if plank.position + plank.dimensions[1]/2. < top_boundary {
                            plank.position += plank.velocity * context.delta_time.num_milliseconds() as f32 / 1000.0;
                        }
                    }
                    // If input_state.right_down == true and plank is in screen boundaries then move down
                    if input_state.right_down {
                        if plank.position - plank.dimensions[1]/2. > bottom_boundary {
                            plank.position -= plank.velocity * context.delta_time.num_milliseconds() as f32 / 1000.0;
                        }
                    }
                    // Set translation[0] of renderable corresponding to this plank
                    renderable.translation[0] = right_boundary - plank.dimensions[0]/2.;
                }
            };
            // Set translation[1] of renderable corresponding to this plank
            renderable.translation[1] = plank.position;
            // Set scale for renderable corresponding to this plank
            renderable.scale[0] = plank.dimensions[0];
            renderable.scale[1] = plank.dimensions[1];
        }

        // Process the ball
        for (ball, renderable) in (&mut balls, &mut renderables).iter() {
            // Move the ball
            ball.position[0] += ball.velocity[0] * context.delta_time.num_milliseconds() as f32 / 1000.0;
            ball.position[1] += ball.velocity[1] * context.delta_time.num_milliseconds() as f32 / 1000.0;

            // Check if the ball has collided with the right plank
            if ball.position[0] + ball.size/2. > right_boundary - left_dimensions[0] &&
               ball.position[0] + ball.size/2. < right_boundary {
                if ball.position[1] - ball.size/2. < right_position + right_dimensions[1]/2. &&
                   ball.position[1] + ball.size/2. > right_position - right_dimensions[1]/2.
                {
                    ball.position[0] = right_boundary - 0.01 - ball.size/2.;
                    ball.velocity[0] = -ball.velocity[0];
                }
            }

            // Check if the ball is to the left of the right boundary, if it is not reset it's position and score the left player
            if ball.position[0] - ball.size/2. > right_boundary {
                ball.position[0] = 0.;
                score.score_left += 1;
                println!("Left player score: {0}, Right player score {1}", score.score_left, score.score_right);
            }

            // Check if the ball has collided with the left plank
            if ball.position[0] - ball.size/2. < left_boundary + left_dimensions[0] &&
               ball.position[0] + ball.size/2. > left_boundary {
                if ball.position[1] - ball.size/2. < left_position + left_dimensions[1]/2. &&
                   ball.position[1] + ball.size/2. > left_position - left_dimensions[1]/2.
                {
                    ball.position[0] = left_boundary + 0.01 + ball.size/2.;
                    ball.velocity[0] = -ball.velocity[0];
                }
            }

            // Check if the ball is to the right of the left boundary, if it is not reset it's position and score the right player
            if ball.position[0] + ball.size/2. < left_boundary {
                ball.position[0] = 0.;
                score.score_right += 1;
                println!("Left player score: {0}, Right player score {1}", score.score_left, score.score_right);
            }

            // Check if the ball is below the top boundary, if it is not deflect it
            if ball.position[1] + ball.size/2. > top_boundary {
                ball.position[1] = top_boundary - ball.size/2.;
                ball.velocity[1] = -ball.velocity[1];
            }

            // Check if the ball is above the bottom boundary, if it is not deflect it
            if ball.position[1] - ball.size/2. < bottom_boundary {
                ball.position[1] = bottom_boundary + ball.size/2.;
                ball.velocity[1] = -ball.velocity[1];
            }

            // Update the renderable corresponding to this ball
            renderable.translation[0] = ball.position[0];
            renderable.translation[1] = ball.position[1];
            renderable.scale[0] = ball.size;
            renderable.scale[1] = ball.size;
        }
    }
}

impl State for Pong {
    fn on_start(&mut self, context: &mut Context, world: &mut World) {

        let (w, h) = context.renderer.get_dimensions().unwrap();

        let aspect = w as f32 / h as f32;

        let eye = [0., 0., 0.1];
        let target = [0., 0., 0.];
        let up = [0., 1., 0.];

        // Get an Orthographic projection
        let projection = Projection::Orthographic {
            left: -1.0 * aspect,
            right: 1.0 * aspect,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1.0,
        };

        // Add all resources
        let input_state = InputState::new();
        let score = Score::new();
        world.add_resource::<InputState>(input_state);
        world.add_resource::<Score>(score);
        world.add_resource::<Projection>(projection.clone());

        // Create a camera entity
        let mut camera = Camera::new(projection, eye, target, up);
        camera.activate();
        world.create_now()
            .with(camera)
            .build();

        // Generate a square mesh
        context.asset_manager.create_constant_texture("white", [1.0, 1.0, 1.0, 1.]);
        context.asset_manager.gen_rectangle("square", 1.0, 1.0);

        // Create a ball entity
        let square = Renderable::new("square", "white", "white");
        let mut ball = Ball::new();
        ball.size = 0.02;
        ball.velocity = [0.5, 0.5];
        world.create_now()
            .with(square)
            .with(ball)
            .build();

        // Create a left plank entity
        let square = Renderable::new("square", "white", "white");
        let mut plank = Plank::new(Side::Left);
        plank.dimensions[0] = 0.01;
        plank.dimensions[1] = 0.1;
        plank.velocity = 1.;
        world.create_now()
            .with(square)
            .with(plank)
            .build();

        // Create right plank entity
        let square = Renderable::new("square", "white", "white");
        let mut plank = Plank::new(Side::Right);
        plank.dimensions[0] = 0.01;
        plank.dimensions[1] = 0.1;
        plank.velocity = 1.;
        world.create_now()
            .with(square)
            .with(plank)
            .build();
    }

    fn update(&mut self, context: &mut Context, _: &mut World) -> Trans {
        // Exit if user hits Escape or closes the window
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let engine_events = context.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }

        Trans::None
    }
}

fn main() {
    use amethyst::engine::Config;
    let config = Config::from_file("../config/pong_example_config.yml").unwrap();
    let mut context = Context::new(config.context_config);
    let rendering_processor = RenderingProcessor::new(config.renderer_config, &mut context);
    let mut game = Application::build(Pong, context)
                   .with::<RenderingProcessor>(rendering_processor, "rendering_processor", 0)

                   .register::<Renderable>()
                   .register::<Light>()
                   .register::<Camera>()

                   .with::<PongProcessor>(PongProcessor, "pong_processor", 0)

                   .register::<Ball>()
                   .register::<Plank>()

                   .done();
    game.run();
}
