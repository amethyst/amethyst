# Adding audio

Now that we have a functional pong game, let's spice things up by adding some audio. In this chapter, we'll add sound effects and background music.

## Adding the Sounds Resource

Let's get started by creating an `audio` subdirectory under `assets`. Then download [the bounce sound][bounce] and [the score sound][score] and put them in `assets/audio`.

Next, we'll create a Resource to store our sound effects in. In `main.rs`, add:

```rust,ignore
mod audio;
```

Create a file called `audio.rs`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
use amethyst::{
    assets::Loader,
    audio::{OggFormat, SourceHandle},
    ecs::{World, WorldExt},
};

const BOUNCE_SOUND: &str = "audio/bounce.ogg";
const SCORE_SOUND: &str = "audio/score.ogg";

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world. This will eventually include
/// the background tracks as well as the sound effects, but for now
/// we'll just work on sound effects.
pub fn initialise_audio(world: &mut World) {
    let sound_effects = {
        let loader = world.read_resource::<Loader>();

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score_sfx: load_audio_track(&loader, &world, SCORE_SOUND),
        };

        sound
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}
```

Then, we'll need to add the Sounds Resource to our World. Update `pong.rs`:

```rust,ignore
use crate::audio::initialise_audio;

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // --snip--

        initialise_audio(world);
    }
}
```

Finally, we'll need our game to include the Audio Bundle. In `main.rs`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::GameDataBuilder;
use amethyst::audio::AudioBundle;

fn main() -> amethyst::Result<()> {
    // --snip--

    let game_data = GameDataBuilder::default()
        // ... other bundles
        .with_bundle(AudioBundle::default())?
        // ... systems
    ;

    // --snip--
# Ok(())
}
```

## Playing the bounce sound

Let's start by creating a function to play the bounce sound. In `audio.rs`, add:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source, SourceHandle},
};
#
# pub struct Sounds {
#     pub score_sfx: SourceHandle,
#     pub bounce_sfx: SourceHandle,
# }
#
pub fn play_bounce_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            output.play_once(sound, 1.0);
        }
    }
}
```

Then, we'll update the Bounce System to play the sound whenever the ball bounces. Update `systems/bounce.rs`:

```rust,ignore

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    ecs::{Read, ReadExpect},
};

use crate::audio::{play_bounce_sound, Sounds};

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &transforms).join() {
            // --snip--

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
            {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                // --snip--

                if point_in_rect(
                    // --snip--
                ) {
                    if (paddle.side == Side::Left && ball.velocity[0] < 0.0)
                        || (paddle.side == Side::Right && ball.velocity[0] > 0.0)
                    {
                        ball.velocity[0] = -ball.velocity[0];
                        play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                    }
                }
            }
        }
    }
}
```

Now try running your game (`cargo run`). Don't forget to turn up your volume!

## Playing the score sound

Just as we did for the bounce sound, let's create a function to play the score sound. Update `audio.rs`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     audio::{output::Output, Source, SourceHandle},
#     assets::AssetStorage,
# };
#
# pub struct Sounds {
#     pub score_sfx: SourceHandle,
#     pub bounce_sfx: SourceHandle,
# }
#
pub fn play_score_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.score_sfx) {
            output.play_once(sound, 1.0);
        }
    }
}
```

Then, we'll update our Winner System to play the score sound whenever a player scores. Update `systems/winner.rs`:

```rust,ignore
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    ecs::Read,
};
use crate::audio::{play_score_sound, Sounds};

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        ReadExpect<'s, ScoreText>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );


    fn run(&mut self, (
        mut balls,
        mut locals,
        mut ui_text,
        mut scores,
        score_text,
        storage,
        sounds,
        audio_output,
    ): Self::SystemData)  {
        for (ball, transform) in (&mut balls, &mut locals).join() {
            // --snip--

            if did_hit {
                ball.velocity[0] = -ball.velocity[0]; // Reverse Direction
                transform.set_translation_x(ARENA_WIDTH / 2.0); // Reset Position
                transform.set_translation_y(ARENA_HEIGHT / 2.0); // Reset Position

                play_score_sound(&*sounds, &storage, audio_output.as_deref());

                // Print the scoreboard.
                println!(
                    "Score: | {:^3} | {:^3} |",
                    scores.score_left, scores.score_right
                );
            }
        }
    }
}
```

Now try running your game. Yay, we successfully added sound effects to our game! 🎉

Next, let's take our game to the next level by adding some background music.

## Adding background music

Let's start by downloading [Albatross][albatross] and [Where's My Jetpack?][wheres-my-jetpack] Put these files in the `assets/audio` directory.

In `audio.rs`, add the paths to the music tracks below the paths to the sound effects:

```rust,edition2018,no_run,noplaypen
const BOUNCE_SOUND: &str = "audio/bounce.ogg";
const SCORE_SOUND: &str = "audio/score.ogg";

const MUSIC_TRACKS: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
```

Then, create a Music Resource:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
use std::{iter::Cycle, vec::IntoIter};
#
# use amethyst::audio::SourceHandle;

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}
```

Since we only have two music tracks, we use a `Cycle` to infinitely alternate between the two.

Next, we need to add the Music Resource to our World. Update `initialise_audio`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use std::{iter::Cycle, vec::IntoIter};
#
use amethyst::{
    audio::{AudioSink, SourceHandle},
    assets::Loader,
    ecs::{World, WorldExt},
};
#
# const BOUNCE_SOUND: &str = "audio/bounce.ogg";
# const SCORE_SOUND: &str = "audio/score.ogg";
#
# const MUSIC_TRACKS: &[&str] = &[
#     "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
#     "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
# ];
#
# fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
#     unimplemented!()
# }
#
# pub struct Music {
#     pub music: Cycle<IntoIter<SourceHandle>>,
# }
#
# pub struct Sounds {
#     pub score_sfx: SourceHandle,
#     pub bounce_sfx: SourceHandle,
# }

pub fn initialise_audio(world: &mut World) {
    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let mut sink = world.write_resource::<AudioSink>();
        sink.set_volume(0.25); // Music is a bit loud, reduce the volume.

        let music = MUSIC_TRACKS
            .iter()
            .map(|file| load_audio_track(&loader, &world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();
        let music = Music { music };

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score_sfx: load_audio_track(&loader, &world, SCORE_SOUND),
        };

        (sound, music)
    };

    // Add sound effects and music to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
    world.insert(music);
}
```

Finally, let's add a DJ System to our game to play the music. In `main.rs`:

```rust,ignore
use amethyst::audio::DjSystemDesc;
use crate::audio::Music;

fn main() -> amethyst::Result<()> {
    // --snip--

    let game_data = GameDataBuilder::default()
        // ... bundles
        .with_system_desc(
            DjSystemDesc::new(|music: &mut Music| music.music.next()),
            "dj_system",
            &[],
        )
        // ... other systems
        ;

    // --snip--
# Ok(())
}
```

Now run your game and enjoy the tunes!

[bounce]: ./audio/bounce.ogg
[score]: ./audio/score.ogg
[albatross]: ./audio/Computer_Music_All-Stars_-_Albatross_v2.ogg
[wheres-my-jetpack]: ./audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg
