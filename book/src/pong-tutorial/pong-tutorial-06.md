# Adding audio

Now that we have a functional pong game, let's spice things up by adding some audio. In this chapter, we'll add sound effects and background music.

## Adding the Sounds Resource

Let's get started by creating an `audio` subdirectory under `assets`. Then download [the bounce sound][bounce] and [the score sound][score] and put them in `assets/audio`.

Next, we'll create a Resource to store our sound effects in. In `main.rs`, add:

```rust,edition2018,no_run,noplaypen
mod audio;

/* ... */

const AUDIO_BOUNCE: &str = "audio/bounce.ogg";
const AUDIO_SCORE: &str = "audio/score.ogg";
```

Create a file called `audio.rs`:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    assets::Loader,
    audio::{OggFormat, SourceHandle},
    ecs::{World, WorldExt},
};

use crate::{AUDIO_BOUNCE, AUDIO_SCORE};

pub struct Sounds {
    pub score: SourceHandle,
    pub bounce: SourceHandle,
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
            bounce: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score: load_audio_track(&loader, &world, SCORE_SOUND),
        };

        sound
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}
```

Then, we'll need to add the Sounds Resource to our World. Update `pong.rs`:

```rust,edition2018,no_run,noplaypen
use crate::{
    /* ... */
    audio::initialise_audio
};

/* ... */

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        /* ... */

        initialise_audio(world);
    }
}
```

Finally, we'll need our game to include the Audio Bundle. In `main.rs`:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    /* ... */
    audio::AudioBundle
};

fn main() -> amethyst::Result<()> {
    /* ... */

    let game_data = GameDataBuilder::default()
        /* ... other bundles */
        .with_bundle(AudioBundle::default())?
        /* ... systems */
    ;

    /* ... */
# Ok(())
}
```

## Playing the bounce sound

Let's start by creating a function to play sounds. In `audio.rs`, add:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use crate::{
    audio::{play_sound, Sounds},
    components::{Ball, Paddle, Side},
    ARENA_HEIGHT, BALL_RADIUS, PADDLE_HEIGHT, PADDLE_WIDTH,
};

const BALL_BOUNDARY_TOP: f32 = ARENA_HEIGHT - BALL_RADIUS;
const BALL_BOUNDARY_BOTTOM: f32 = BALL_RADIUS;

#[derive(SystemDesc)]
pub struct BounceSystem;

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
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= BALL_BOUNDARY_BOTTOM && ball.heads_down())
                || (ball_y >= BALL_BOUNDARY_TOP && ball.heads_up())
            {
                ball.reverse_y()
                play_sound(&sounds.bounce, &storage, audio_output.as_deref());
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width / 2.0);
                let paddle_y = paddle_transform.translation().y - (paddle.height / 2.0);

                if point_in_rect(ball_x, ball_y, paddle_x, paddle_y)
                    && ((paddle.side == Side::Left && ball.heads_left())
                        || (paddle.side == Side::Right && ball.heads_right()))
                {
                    ball.reverse_x();
                    play_sound(&sounds.bounce, &storage, audio_output.as_deref());
                }
            }
        }
    }
}

fn point_in_rect(ball_x: f32, ball_y: f32, paddle_x: f32, paddle_y: f32) -> bool {
    /* ... */
}
```
Now try running your game (`cargo run`). Don't forget to turn up your volume!

## Playing the score sound

Let's update our Winner System to play the score sound whenever a player scores. Update `systems/winner.rs`:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadExpect, System, SystemData, Write, WriteStorage},
    ui::UiText,
};

use crate::{
    audio::{play_sound, Sounds},
    components::Ball,
    pong::{ScoreBoard, ScoreText},
    ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS,
};

const BALL_BOUNDARY_RIGHT: f32 = ARENA_HEIGHT - BALL_RADIUS;
const BALL_BOUNDARY_LEFT: f32 = BALL_RADIUS;

/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        ReadExpect<'s, ScoreText>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (
            mut balls,
            mut transforms,
            mut text,
            mut score_board,
            storage,
            sounds,
            score_text,
            audio_output,
        ): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            /* ... */

            if scored {
                /* ... */

                // Play audio.
                play_sound(&sounds.score, &storage, audio_output.as_deref());
            }
        }
    }
}

```
Now try running your game. We successfully added sound effects to our game! 🎉

Next, let's take our game to the next level by adding some background music.

## Adding background music

Let's start by downloading [Albatross][albatross] and [Where's My Jetpack?][wheres-my-jetpack] Put these files in the `assets/audio` directory.

In `main.rs`, add the paths to the music tracks:

```rust,edition2018,no_run,noplaypen
const MUSIC_TRACKS: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
```

And use them in `audio.rs`:
```rust,edition2018,no_run,noplaypen
use crate::{MUSIC_TRACKS, /* ... */}
```

Then, create a Music Resource:

```rust,edition2018,no_run,noplaypen
use std::{iter::Cycle, vec::IntoIter};

/* ... */

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}
```

Since we only have two music tracks, we use a `Cycle` to infinitely alternate between the two.

Next, we need to add the Music Resource to our World. Update `initialise_audio`:

```rust,edition2018,no_run,noplaypen
use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, AudioSink, OggFormat, Source, SourceHandle},
    ecs::{World, WorldExt},
};
#
# use std::{iter::Cycle, vec::IntoIter};

/* ... */

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
            bounce: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score: load_audio_track(&loader, &world, SCORE_SOUND),
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

```rust,edition2018,no_run,noplaypen
use amethyst::audio::DjSystemDesc;

use crate::audio::Music;

fn main() -> Result<()> {
    /* ... */

    let game_data = GameDataBuilder::default()
        /* ... other bundles */
        .with_system_desc(
            DjSystemDesc::new(|music: &mut Music| music.music.next()),
            "dj_system",
            &[],
        )
        /* ... other systems */
        ;

    /* ... */
# Ok(())
}
```

Now run your game and enjoy the tunes!

[bounce]: ./audio/bounce.ogg
[score]: ./audio/score.ogg
[albatross]: ./audio/Computer_Music_All-Stars_-_Albatross_v2.ogg
[wheres-my-jetpack]: ./audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg