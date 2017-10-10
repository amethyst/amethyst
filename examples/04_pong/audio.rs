use amethyst::audio::{AudioContext, Dj, Source};
use amethyst::assets::Loader;
use amethyst::ecs::World;
use futures::prelude::*;

pub struct Sounds {
    pub score_sfx: Source,
    pub bounce_sfx: Source,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &mut Loader, file: &str) -> Source {
    use amethyst::assets::formats::audio::OggFormat;

    loader
        .load_from(file, OggFormat, "assets")
        .wait()
        .expect(&format!("Could not load audio track: '{}'", file))
}

/// Initialise audio in the world. This includes the background track and the
/// sound effects.
pub fn initialise_audio(world: &mut World) {
    use {AUDIO_BOUNCE, AUDIO_MUSIC, AUDIO_SCORE};
    use amethyst::audio::output::Output;

    let sound_effects = {
        let mut loader = world.write_resource::<Loader>();
        loader.register(AudioContext::new());

        // Add a DJ if we have sound output and background music tracks.
        if world.read_resource::<Option<Output>>().is_some() && AUDIO_MUSIC.len() > 0 {
            let mut dj = world.write_resource::<Dj>();
            dj.set_volume(0.25); // Music is a bit loud, reduce the volume.
            let mut next_track_index = 0;

            let music_tracks: Vec<_> = AUDIO_MUSIC
                .iter()
                .map(|file| load_audio_track(&mut loader, file))
                .collect();

            dj.set_picker(Box::new(move |ref mut dj| {
                dj.append(&music_tracks[next_track_index])
                    .expect("Decoder error occurred!");
                next_track_index = (next_track_index + 1) % music_tracks.len();
                true
            }));
        }

        Sounds {
            bounce_sfx: load_audio_track(&mut loader, AUDIO_BOUNCE),
            score_sfx: load_audio_track(&mut loader, AUDIO_SCORE),
        }
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.add_resource(sound_effects);
}
