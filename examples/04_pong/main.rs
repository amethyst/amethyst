//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate amethyst_renderer;
extern crate cgmath;
extern crate futures;
extern crate rayon;

use std::sync::Arc;

use amethyst::assets::Loader;
use amethyst::assets::formats::audio::OggFormat;
use amethyst::audio::{Dj, AudioContext, Source};
use amethyst::audio::output::{default_output, Output};
use amethyst::audio::play::play_once;
use amethyst::prelude::*;
use amethyst::assets::{AssetFuture, BoxedErr};
use amethyst::ecs::{Component, Fetch, FetchMut, Join, System, VecStorage, WriteStorage};
use amethyst::ecs::transform::{Transform, LocalTransform, Child, Init, TransformSystem};
use amethyst::ecs::rendering::{Factory, MeshComponent, MaterialComponent};
use amethyst::ecs::input::{Bindings, InputHandler};
use amethyst::ecs::audio::DjSystem;
use amethyst::timing::Time;
use amethyst_renderer::prelude::*;
use amethyst_renderer::Config as DisplayConfig;

use futures::{Future, IntoFuture};

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

struct PongSystem {
    score_sfx: Source,
    bounce_sfx: Source,
    audio_output: Option<Output>,
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
           (mut balls, mut planks, mut locals, _, time, input, mut score): Self::SystemData) {
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
                    local.translation[0] = plank.dimensions[0] / 2.0
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
                    local.translation[0] = 1.0 - plank.dimensions[0] / 2.0
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
            if ball.position[0] + ball.size / 2. > 1.0 - left_dimensions[0] &&
               ball.position[0] + ball.size / 2. < 1.0 {
                if ball.position[1] - ball.size / 2. < right_position + right_dimensions[1] / 2. &&
                   ball.position[1] + ball.size / 2. > right_position - right_dimensions[1] / 2. {
                    ball.position[0] = 1.0 - right_dimensions[0] - ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                    if let Some(ref output) = self.audio_output {
                        play_once(&self.bounce_sfx, &output);
                    }
                }
            }

            // Check if the ball is to the left of the right boundary
            // if it is not reset it's position and score the left player
            if ball.position[0] - ball.size / 2. > 1.0 {
                ball.position[0] = 0.5;
                score.score_left += 1;
                println!("Left player score: {0}, Right player score {1}",
                         score.score_left,
                         score.score_right);
                if let Some(ref output) = self.audio_output {
                    play_once(&self.score_sfx, &output);
                }
            }

            // Check if the ball has collided with the left plank
            if ball.position[0] - ball.size / 2. < left_dimensions[0] &&
               ball.position[0] + ball.size / 2. > 0.0 {
                if ball.position[1] - ball.size / 2. < left_position + left_dimensions[1] / 2. &&
                   ball.position[1] + ball.size / 2. > left_position - left_dimensions[1] / 2. {
                    ball.position[0] = left_dimensions[0] + ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                    if let Some(ref output) = self.audio_output {
                        play_once(&self.bounce_sfx, &output);
                    }
                }
            }

            // Check if the ball is to the right of the left boundary
            // if it is not reset it's position and score the right player
            if ball.position[0] + ball.size / 2. < 0.0 {
                ball.position[0] = 0.5;
                score.score_right += 1;
                println!("Left player score: {0}, Right player score {1}",
                         score.score_left,
                         score.score_right);
                if let Some(ref output) = self.audio_output {
                    play_once(&self.score_sfx, &output);
                }
            }

            // Check if the ball is below the top boundary, if it is not deflect it
            if ball.position[1] + ball.size / 2. > 1.0 {
                ball.position[1] = 1.0 - ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
                if let Some(ref output) = self.audio_output {
                    play_once(&self.bounce_sfx, &output);
                }
            }

            // Check if the ball is above the bottom boundary, if it is not deflect it
            if ball.position[1] - ball.size / 2. < 0.0 {
                ball.position[1] = ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
                if let Some(ref output) = self.audio_output {
                    play_once(&self.bounce_sfx, &output);
                }
            }

            // Update the renderable corresponding to this ball
            local.translation[0] = ball.position[0];
            local.translation[1] = ball.position[1];
            local.scale[0] = ball.size;
            local.scale[1] = ball.size;
        }
    }
}

fn load_proc_asset<T, F>(engine: &mut Engine, f: F) -> AssetFuture<T::Item>
    where T: IntoFuture<Error=BoxedErr>,
          T::Future: 'static,
          F: FnOnce(&mut Engine) -> T
{
    let future = f(engine).into_future();
    let future: Box<Future<Item=T::Item, Error=BoxedErr>> = Box::new(future);
    AssetFuture(future.shared())
}

impl State for Pong {
    fn on_start(&mut self, engine: &mut Engine) {

        // Generate a square mesh
        let tex = Texture::from_color_val([1.0, 1.0, 1.0, 1.0]);
        let mtl = MaterialBuilder::new().with_albedo(tex);

        let square_verts = gen_rectangle(1.0, 1.0);
        let mesh = Mesh::build(square_verts);

        let mesh = load_proc_asset(engine, move |engine| {
            let factory = engine.world.read_resource::<Factory>();
            factory
                .create_mesh(mesh)
                .map(MeshComponent::new)
                .map_err(BoxedErr::new)
        });

        let mtl = load_proc_asset(engine, move |engine| {
            let factory = engine.world.read_resource::<Factory>();
            factory
                .create_material(mtl)
                .map(MaterialComponent)
                .map_err(BoxedErr::new)
        });


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

        world.register::<Child>();
        world.register::<Init>();
        world.register::<LocalTransform>();

        // Create a ball entity
        let mut ball = Ball::new();
        ball.size = 0.02;
        ball.velocity = [0.5, 0.5];
        world
            .create_entity()
            .with(mesh.clone())
            .with(mtl.clone())
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
            .with(mesh.clone())
            .with(mtl.clone())
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
            .with(mesh)
            .with(mtl)
            .with(plank)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }

    fn handle_event(&mut self, engine: &mut Engine, event: Event) -> Trans {
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
                    Trans::None
                }
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    use futures::future::Future;
    use rayon::{Configuration, ThreadPool};

    let path = format!("{}/examples/04_pong/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let assets_dir = format!("{}/examples/04_pong/resources/",
                                            env!("CARGO_MANIFEST_DIR"));
    let mut loader = Loader::new(assets_dir, Arc::new(ThreadPool::new(Configuration::new()).unwrap()));
    loader.register(AudioContext::new());
    let bounce_sfx = loader.load("bounce",
                                OggFormat)
        .wait().unwrap();
    let score_sfx = loader.load("score",
                               OggFormat)
        .wait().unwrap();
    let music_1: Source = loader.load("Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
                               OggFormat)
        .wait().unwrap();
    let music_2: Source = loader.load("Computer_Music_All-Stars_-_Albatross_v2.ogg",
                               OggFormat)
        .wait().unwrap();
    let audio_output = default_output();
    let dj = match audio_output {
        Some(ref output) => {
            let mut dj = Dj::new(&output);
            dj.set_volume(0.25); // Music is a bit loud, reduce the volume.
            let mut playing_1 = false;
            let music_1 = music_1.clone();
            let music_2 = music_2.clone();
            dj.set_picker(Box::new(move |ref mut dj| {
                if playing_1 {
                    dj.append(&music_2).expect("Decoder error occurred!");
                    playing_1 = false;
                } else {
                    dj.append(&music_1).expect("Decoder error occurred!");
                    playing_1 = true;
                }
                true
            }));
            Some(dj)
        }
        None => {
            eprintln!("Audio device not found, no sound will be played.");
            None
        }
    };
    let pong = PongSystem {
        bounce_sfx: bounce_sfx,
        score_sfx: score_sfx,
        audio_output: audio_output,
    };
    let mut game = Application::build(Pong).unwrap()
        .register::<Ball>()
        .register::<Plank>()
        .with::<PongSystem>(pong, "pong_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &["pong_system"])
        .with_renderer(Pipeline::build()
                           .with_stage(Stage::with_backbuffer()
                               .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                               .with_model_pass(pass::DrawFlat::<PosNormTex>::new())
                           ),
                       Some(cfg)
        )?;
    if let Some(dj) = dj {
        game = game.add_resource(dj)
            .with(DjSystem, "dj_system", &[]);
    }
    Ok(game.build()?.run())
}



fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
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
