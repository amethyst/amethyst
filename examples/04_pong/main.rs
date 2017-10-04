//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate futures;

use amethyst::{ApplicationBuilder, Result};
use amethyst::assets::{AssetFuture, BoxedErr};
use amethyst::assets::Loader;
use amethyst::assets::formats::audio::OggFormat;
use amethyst::audio::{AudioContext, Dj, Source};
use amethyst::audio::output::Output;
use amethyst::audio::play::play_once;
use amethyst::ecs::{Component, DenseVecStorage, ECSBundle, Fetch, FetchMut, Join, System,
                    WriteStorage};
use amethyst::ecs::audio::DjBundle;
use amethyst::ecs::input::{InputBundle, InputHandler};
use amethyst::ecs::rendering::{Factory, MaterialComponent, MeshComponent, RenderBundle};
use amethyst::ecs::rendering::resources::WindowMessages;
use amethyst::ecs::transform::{LocalTransform, Transform, TransformBundle};
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::prelude::*;
use amethyst::timing::Time;
use amethyst::winit::CursorState;
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
    type Storage = DenseVecStorage<Self>;
}

enum Side {
    Left,
    Right,
}

struct Paddle {
    pub position: f32,
    pub velocity: f32,
    pub dimensions: [f32; 2],
    pub side: Side,
}

impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            position: 0.,
            velocity: 1.,
            dimensions: [1., 1.],
            side: side,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

struct Sounds {
    score_sfx: Source,
    bounce_sfx: Source,
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

struct PongBundle;

impl<'a, 'b, T> ECSBundle<'a, 'b, T> for PongBundle {
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        Ok(
            builder
                .with_resource(Score::new())
                .with_resource(Time::default())
                .register::<Ball>()
                .register::<Paddle>()
                .with(PongSystem, "pong_system", &["input_system"]),
        )
    }
}

// Pong game system
impl<'a> System<'a> for PongSystem {
    type SystemData = (
        WriteStorage<'a, Ball>,
        WriteStorage<'a, Paddle>,
        WriteStorage<'a, LocalTransform>,
        Fetch<'a, Camera>,
        Fetch<'a, Time>,
        Fetch<'a, InputHandler<String, String>>,
        Fetch<'a, Sounds>,
        Fetch<'a, Option<Output>>,
        FetchMut<'a, Score>,
    );

    fn run(
        &mut self,
        (mut balls, mut planks, mut locals, _, time, input, sounds, audio_output, mut score):
        Self::SystemData,
    )
    {
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
            if ball.position[0] + ball.size / 2. > 1.0 - left_dimensions[0]
                && ball.position[0] + ball.size / 2. < 1.0
            {
                if ball.position[1] - ball.size / 2. < right_position + right_dimensions[1] / 2.
                    && ball.position[1] + ball.size / 2. > right_position - right_dimensions[1] / 2.
                {
                    ball.position[0] = 1.0 - right_dimensions[0] - ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                    if let Some(ref output) = *audio_output {
                        play_once(&sounds.bounce_sfx, 1.0, &output);
                    }
                }
            }

            // Check if the ball is to the left of the right boundary
            // if it is not reset it's position and score the left player
            if ball.position[0] - ball.size / 2. > 1.0 {
                ball.position[0] = 0.5;
                score.score_left += 1;
                println!(
                    "Left player score: {0}, Right player score {1}",
                    score.score_left,
                    score.score_right
                );
                if let Some(ref output) = *audio_output {
                    play_once(&sounds.score_sfx, 1.0, &output);
                }
            }

            // Check if the ball has collided with the left plank
            if ball.position[0] - ball.size / 2. < left_dimensions[0]
                && ball.position[0] + ball.size / 2. > 0.0
            {
                if ball.position[1] - ball.size / 2. < left_position + left_dimensions[1] / 2.
                    && ball.position[1] + ball.size / 2. > left_position - left_dimensions[1] / 2.
                {
                    ball.position[0] = left_dimensions[0] + ball.size / 2.;
                    ball.velocity[0] = -ball.velocity[0];
                    if let Some(ref output) = *audio_output {
                        play_once(&sounds.bounce_sfx, 1.0, &output);
                    }
                }
            }

            // Check if the ball is to the right of the left boundary
            // if it is not reset it's position and score the right player
            if ball.position[0] + ball.size / 2. < 0.0 {
                ball.position[0] = 0.5;
                score.score_right += 1;
                println!(
                    "Left player score: {0}, Right player score {1}",
                    score.score_left,
                    score.score_right
                );
                if let Some(ref output) = *audio_output {
                    play_once(&sounds.score_sfx, 1.0, &output);
                }
            }

            // Check if the ball is below the top boundary, if it is not deflect it
            if ball.position[1] + ball.size / 2. > 1.0 {
                ball.position[1] = 1.0 - ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
                if let Some(ref output) = *audio_output {
                    play_once(&sounds.bounce_sfx, 1.0, &output);
                }
            }

            // Check if the ball is above the bottom boundary, if it is not deflect it
            if ball.position[1] - ball.size / 2. < 0.0 {
                ball.position[1] = ball.size / 2.;
                ball.velocity[1] = -ball.velocity[1];
                if let Some(ref output) = *audio_output {
                    play_once(&sounds.bounce_sfx, 1.0, &output);
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
where
    T: IntoFuture<Error = BoxedErr>,
    T::Future: 'static,
    F: FnOnce(&mut Engine) -> T,
{
    let future = f(engine).into_future();
    let future: Box<Future<Item = T::Item, Error = BoxedErr>> = Box::new(future);
    AssetFuture(future.shared())
}

impl State for Pong {
    fn on_start(&mut self, engine: &mut Engine) {
        // Hide the cursor
        engine
            .world
            .write_resource::<WindowMessages>()
            .send_command(|win| {
                if let Err(err) = win.set_cursor_state(CursorState::Hide) {
                    eprintln!("Unable to make cursor hidden! Error: {:?}", err);
                }
            });
        // Load audio assets
        // FIXME: do loading with futures, pending the Loading state
        {
            let (music_1, music_2, bounce_sfx, score_sfx) = {
                let mut loader = engine.world.write_resource::<Loader>();
                loader.register(AudioContext::new());

                let music_1: Source = loader
                    .load_from(
                        "Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
                        OggFormat,
                        "assets",
                    )
                    .wait()
                    .unwrap();

                let music_2: Source = loader
                    .load_from(
                        "Computer_Music_All-Stars_-_Albatross_v2.ogg",
                        OggFormat,
                        "assets",
                    )
                    .wait()
                    .unwrap();

                let bounce_sfx = loader
                    .load_from("bounce", OggFormat, "assets")
                    .wait()
                    .unwrap();
                let score_sfx = loader
                    .load_from("score", OggFormat, "assets")
                    .wait()
                    .unwrap();

                (music_1, music_2, bounce_sfx, score_sfx)
            };

            engine.world.add_resource(Sounds {
                bounce_sfx,
                score_sfx,
            });

            let have_output = engine.world.read_resource::<Option<Output>>().is_some();

            if have_output {
                let mut dj = engine.world.write_resource::<Dj>();
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
            }
        }

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
        let mut plank = Paddle::new(Side::Left);
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
        let mut plank = Paddle::new(Side::Right);
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

type DrawFlat = pass::DrawFlat<PosNormTex, MeshComponent, MaterialComponent, Transform>;

fn run() -> Result<()> {
    use amethyst::assets::Directory;

    let path = format!(
        "{}/examples/04_pong/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let bindings_path = format!(
        "{}/examples/04_pong/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let cfg = DisplayConfig::load(path);
    let assets_dir = format!("{}/examples/04_pong/resources/", env!("CARGO_MANIFEST_DIR"));
    let game = Application::build(Pong)?
        .with_bundle(InputBundle::<String, String>::new().with_bindings_from_file(&bindings_path))?
        .with_bundle(PongBundle)?
        .with_bundle(TransformBundle::new().with_dep(&["input_system", "pong_system"]))?
        .with_bundle(DjBundle::new())?
        .with_bundle(
            RenderBundle::new(
                Pipeline::build().with_stage(
                    Stage::with_backbuffer()
                        .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                        .with_pass(DrawFlat::new()),
                ),
            ).with_config(cfg),
        )?
        .with_store("assets", Directory::new(assets_dir));
    Ok(game.build()?.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn gen_rectangle(w: f32, h: f32) -> Vec<PosNormTex> {
    let data: Vec<PosNormTex> = vec![
        PosNormTex {
            position: [-w / 2., -h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [0., 0.],
        },
        PosNormTex {
            position: [w / 2., -h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [1., 0.],
        },
        PosNormTex {
            position: [w / 2., h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [1., 1.],
        },
        PosNormTex {
            position: [w / 2., h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [1., 1.],
        },
        PosNormTex {
            position: [-w / 2., h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [0., 1.],
        },
        PosNormTex {
            position: [-w / 2., -h / 2., 0.],
            normal: [0., 0., 1.],
            tex_coord: [0., 0.],
        },
    ];
    data
}
