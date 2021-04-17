# Adding audio

Now that we have a functional pong game, let's spice things up by adding some audio. In this chapter, we'll add sound effects and background music.

## Adding the Sounds Resource

Let's get started by creating an `audio` subdirectory under `assets`. Then download [the bounce sound][bounce] and [the score sound][score] and put them in `assets/audio`.

Next, we'll create a Resource to store our sound effects in. In `main.rs`, add:

```rust
mod audio;
```

Create a file called `audio.rs`:

```rust
use amethyst::{
    assets::{AssetStorage, DefaultLoader, Loader},
    audio::{
        output::{Output, OutputWrapper},
        Source, SourceHandle,
    },
    ecs::{Resources, World},
};

const AUDIO_BOUNCE: &str = "audio/bounce.ogg";
const AUDIO_SCORE: &str = "audio/score.ogg";

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &DefaultLoader, file: &str) -> SourceHandle {
    loader.load(file)
}

/// initialize audio in the world. This will eventually include
/// the background tracks as well as the sound effects, but for now
/// we'll work on sound effects.
pub fn initialize_audio(world: &mut World) {
    let sound_effects = {
        let loader = resources.get::<DefaultLoader>().unwrap();

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, AUDIO_BOUNCE),
            score_sfx: load_audio_track(&loader, AUDIO_SCORE),
        };

        sound
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    resources.insert(sound_effects);
}
```

Then, we'll need to add the Sounds Resource to our World. Update `pong.rs`:

```rust
use crate::audio::initialize_audio;

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        // --snip--

        initialize_audio(world);
    }
}
```

Finally, we'll need our game to include the Audio Bundle. In `main.rs`:

```rust
# use amethyst::DispatcherBuilder;
use amethyst::audio::AudioBundle;

fn main() -> amethyst::Result<()> {
    // --snip--

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
// ... other bundles
.add_bundle(AudioBundle)
// ... systems
;

    // --snip--
#   Ok(())
}
```

## Playing the bounce sound

Let's start by creating a function to play the bounce sound. In `audio.rs`, add:

```rust
use amethyst::{
    assets::{AssetStorage, DefaultLoader, Loader},
    audio::{
        output::{Output, OutputWrapper},
        Source, SourceHandle,
    },
    ecs::{Resources, World},
};
# pub struct Sounds {
#   pub score_sfx: SourceHandle,
#   pub bounce_sfx: SourceHandle,
# }
# 
/// Plays the bounce sound when a ball hits a side or a paddle.
pub fn play_bounce(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            output.play_once(sound, 1.0);
        }
    }
}
```

Then, we'll update the Bounce System to play the sound whenever the ball bounces. Update `systems/bounce.rs`:

```rust
use amethyst::{
    assets::AssetStorage,
    audio::{output::OutputWrapper, Source},
    core::transform::Transform,
    ecs::SystemBuilder,
    prelude::*,
};

use crate::{
    audio::{play_bounce, Sounds},
    pong::{Ball, Paddle, Side, ARENA_HEIGHT},
};

pub struct BounceSystem;

impl System for BounceSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("BounceSystem")
#               .read_resource::<Sounds>()
#               .read_resource::<AssetStorage<Source>>()
#               .read_resource::<OutputWrapper>()
#               .with_query(<(&mut Ball, &Transform)>::query())
#               .with_query(<&Paddle>::query())
#               .read_component::<Paddle>()
#               .read_component::<Transform>()
#               .write_component::<Ball>()
                .build(
                    move |_commands,
                          world,
                          (sounds, storage, output_wrapper),
                          (query_balls, query_paddles)| {
                        let (mut ball_world, remaining) = world.split_for_query(query_balls);

                        for (ball, transform) in query_balls.iter_mut(&mut ball_world) {
#                           let ball_x = transform.translation().x;
#                           let ball_y = transform.translation().y;

#                           // Bounce at the top or the bottom of the arena.
#                           if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
#                               || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
#                           {
#                               ball.velocity[1] = -ball.velocity[1];
#                           }

                            // -- snip --
                            for paddle in query_paddles.iter(&remaining) {
#                               if point_in_rect(
#                                   ball_x,
#                                   ball_y,
#                                   paddle.x - paddle.width / 2. - ball.radius,
#                                   paddle.y - paddle.height / 2. - ball.radius,
#                                   paddle.x + paddle.width / 2. + ball.radius,
#                                   paddle.y + paddle.height / 2. + ball.radius,
#                               ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0)
#                                   || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                            // -- snip --
                                {
                                    println!("Bounce!");
                                    play_bounce(sounds, storage, output_wrapper.output.as_ref());
                                    ball.velocity[0] = -ball.velocity[0];
                                }
                            }
                        }
                    },
                ),
        )
    }
}

# fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
#     x >= left && x <= right && y >= bottom && y <= top
# }

```

Now try running your game (`cargo run`). Don't forget to turn up your volume!

## Playing the score sound

Just as we did for the bounce sound, let's create a function to play the score sound. Update `audio.rs`:

```rust
# use amethyst::{
#   assets::AssetStorage,
#   audio::{output::Output, Source, SourceHandle},
# };
# 
# pub struct Sounds {
#   pub score_sfx: SourceHandle,
#   pub bounce_sfx: SourceHandle,
# }
# 
pub fn play_score(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.score_sfx) {
            output.play_once(sound, 1.0);
        }
    }
}
```

Then, we'll update our Winner System to play the score sound whenever a player scores. Update `systems/winner.rs`:

```rust
use amethyst::{
    assets::AssetStorage,
    audio::{output::OutputWrapper, Source},
    core::{
        ecs::{ParallelRunnable, System},
        transform::Transform,
    },
    ecs::{IntoQuery, SystemBuilder},
    ui::UiText,
};

use crate::audio::{play_score, Sounds};
use crate::pong::{Ball, ScoreBoard, ScoreText, ARENA_HEIGHT, ARENA_WIDTH};

pub struct WinnerSystem;

impl System for WinnerSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("WinnerSystem")
                .with_query(<(&mut Ball, &mut Transform)>::query())
                .with_query(<&mut UiText>::query())
                .write_component::<Ball>()
                .write_component::<Transform>()
                .write_component::<UiText>()
                .write_resource::<ScoreBoard>()
                .read_resource::<ScoreText>()
                .read_resource::<Sounds>()
                .read_resource::<AssetStorage<Source>>()
                .read_resource::<OutputWrapper>()
                .build(
#                   move |_commands,
#                         world,
#                         (score_board, score_text, sounds, storage, output_wrapper),
#                         (balls_query, edit_query)| {
#                       let (mut ball_world, mut score_world) = world.split_for_query(balls_query);
#
#                       for (ball, transform) in balls_query.iter_mut(&mut ball_world) {
#                           let ball_x = transform.translation().x;
#                           let did_hit = if ball_x <= ball.radius {
#                               // Right player scored on the left side.
#                               // We top the score at 999 to avoid text overlap.
#                               score_board.score_right = (score_board.score_right + 1).min(999);
#                               if let Ok(text) =
#                                   edit_query.get_mut(&mut score_world, score_text.p2_score)
#                               {
#                                   text.text = score_board.score_right.to_string();
#                               }
#                               true
#                           } else if ball_x >= ARENA_WIDTH - ball.radius {
#                               // Left player scored on the right side.
#                               // We top the score at 999 to avoid text overlap.
#                               score_board.score_left = (score_board.score_left + 1).min(999);
#                               if let Ok(text) =
#                                   edit_query.get_mut(&mut score_world, score_text.p1_score)
#                               {
#                                   text.text = score_board.score_left.to_string();
#                               }
#                               true
#                           } else {
#                               false
#                           };
                            //  -- snip --
                            if did_hit {
                                // Reset the ball.
                                ball.velocity[0] = -ball.velocity[0];
                                transform.set_translation_x(ARENA_WIDTH / 2.0);
                                transform.set_translation_y(ARENA_HEIGHT / 2.0);

                                play_score(&*sounds, &storage, output_wrapper.output.as_ref());
                                // Print the score board.
                                println!(
                                    "Score: | {:^3} | {:^3} |",
                                    score_board.score_left, score_board.score_right
                                );
                            }
                        }
                    },
                ),
        )
    }
}

```

Now try running your game. Yay, we successfully added sound effects to our game! 🎉

Next, let's take our game to the next level by adding some background music.

## Adding background music

Let's start by downloading [Albatross] and [Where's My Jetpack?][wheres-my-jetpack] Put these files in the `assets/audio` directory.

In `audio.rs`, add the paths to the music tracks by the paths to the sound effects:

```rust
const AUDIO_MUSIC: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &str = "audio/bounce.ogg";
const AUDIO_SCORE: &str = "audio/score.ogg";
```

Then, create a Music Resource:

```rust
use std::{iter::Cycle, vec::IntoIter};
# use amethyst::audio::SourceHandle;

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}
```

Since we only have two music tracks, we use a `Cycle` to infinitely alternate between the two.

Next, we need to add the Music Resource to our World. Update `initialize_audio`:

```rust
# use std::{iter::Cycle, vec::IntoIter};
# 
use amethyst::{
    assets::Loader,
    audio::{AudioSink, SourceHandle},
    ecs::World,
};
# const BOUNCE_SOUND: &str = "audio/bounce.ogg";
# const SCORE_SOUND: &str = "audio/score.ogg";
# 
# const MUSIC_TRACKS: &[&str] = &[
#   "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
#   "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
# ];
# 
# fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
#   unimplemented!()
# }
# 
# pub struct Music {
#   pub music: Cycle<IntoIter<SourceHandle>>,
# }
# 
# pub struct Sounds {
#   pub score_sfx: SourceHandle,
#   pub bounce_sfx: SourceHandle,
# }

pub fn initialize_audio(_: &mut World, resources: &mut Resources) {
    let (sound_effects, music) = {
        let loader = resources.get::<DefaultLoader>().unwrap();

        // Music is a bit loud, reduce the volume.
        resources
            .get_mut::<OutputWrapper>()
            .unwrap()
            .audio_sink
            .as_mut()
            .unwrap()
            .set_volume(0.25);

        let music = AUDIO_MUSIC
            .iter()
            .map(|file| load_audio_track(&loader, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();
        let music = Music { music };

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, AUDIO_BOUNCE),
            score_sfx: load_audio_track(&loader, AUDIO_SCORE),
        };

        (sound, music)
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    resources.insert(sound_effects);
    resources.insert(music);
}
```

Finally, let's add a DJ System to our game to play the music. In `main.rs`:

```rust
use crate::audio::Music;
use amethyst::audio::DjSystemDesc;

fn main() -> amethyst::Result<()> {
    // --snip--

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
// ... bundles
.add_bundle(DjSystemBundle::new(|music: &mut Music| music.music.next()))
// ... other systems
;

    // --snip--
#   Ok(())
}
```

Now run your game and enjoy the tunes!

[albatross]: ./audio/Computer_Music_All-Stars_-_Albatross_v2.ogg
[bounce]: ./audio/bounce.ogg
[score]: ./audio/score.ogg
[wheres-my-jetpack]: ./audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg
