use std::{iter::Cycle, vec::IntoIter};

use amethyst::{
    assets::{AssetStorage, DefaultLoader, Loader},
    audio::{
        output::{Output, OutputWrapper},
        Source, SourceHandle,
    },
    ecs::{Resources, World},
};

const AUDIO_MUSIC: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &str = "audio/bounce.ogg";
const AUDIO_SCORE: &str = "audio/score.ogg";

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &DefaultLoader, file: &str) -> SourceHandle {
    loader.load(file)
}

/// initialize audio in the world. This includes the background track and the
/// sound effects.
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

/// Plays the bounce sound when a ball hits a side or a paddle.
pub fn play_bounce(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            output.play_once(sound, 1.0);
        }
    }
}
