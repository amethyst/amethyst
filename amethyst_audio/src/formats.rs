use amethyst_assets::Format;
use amethyst_error::Error;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "caa6e38f-9cfa-428a-91bd-4dab5a7a47d5"]
pub struct AudioData(pub Vec<u8>);
amethyst_assets::register_asset_type!(AudioData => crate::Source; amethyst_assets::AssetProcessorSystem<crate::Source>);

/// Loads audio from wav files.
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "e78ea33f-d506-4d4f-8276-861660bb6145"]
pub struct WavFormat;

amethyst_assets::register_importer!(".wav", WavFormat);
impl Format<AudioData> for WavFormat {
    fn name(&self) -> &'static str {
        "WAV"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Ogg Vorbis files
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "8ce12d56-9091-4e25-b764-da162fa165aa"]
pub struct OggFormat;

amethyst_assets::register_importer!(".ogg", OggFormat);
impl Format<AudioData> for OggFormat {
    fn name(&self) -> &'static str {
        "OGG"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Flac files.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "15522fa0-9996-4416-840f-1e99c7a31f1a"]
pub struct FlacFormat;

amethyst_assets::register_importer!(".flac", FlacFormat);
impl Format<AudioData> for FlacFormat {
    fn name(&self) -> &'static str {
        "FLAC"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from MP3 files.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "f693f1ec-e148-4190-b6ac-e3dc9795031c"]
pub struct Mp3Format;

amethyst_assets::register_importer!(".mp3", Mp3Format);
impl Format<AudioData> for Mp3Format {
    fn name(&self) -> &'static str {
        "MP3"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}
