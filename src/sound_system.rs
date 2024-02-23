use log::warn;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::sync::Arc;

//
// Thanks Bevy.
// AudioOutput and AudioSource are from Bevy.
//

/// Used internally to play audio on the current "audio device"
///
/// ## Note
///
/// Initializing this resource will leak [`OutputStream`]
/// using [`std::mem::forget`].
/// This is done to avoid storing this in the struct (and making this `!Send`)
/// while preventing it from dropping (to avoid halting of audio).
///
/// This is fine when initializing this once (as is default when adding this plugin),
/// since the memory cost will be the same.
/// However, repeatedly inserting this resource into the app will **leak more memory**.
pub struct AudioOutput {
    pub stream_handle: Option<OutputStreamHandle>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            // We leak `OutputStream` to prevent the audio from stopping.
            std::mem::forget(stream);
            Self {
                stream_handle: Some(stream_handle),
            }
        } else {
            warn!("No audio device found.");
            Self { stream_handle: None }
        }
    }
}
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Raw data of the audio source.
    ///
    /// The data must be one of the file formats supported by Bevy (`wav`, `ogg`, `flac`, or `mp3`).
    /// It is decoded using [`rodio::decoder::Decoder`](https://docs.rs/rodio/latest/rodio/decoder/struct.Decoder.html).
    ///
    /// The decoder has conditionally compiled methods
    /// depending on the features enabled.
    /// If the format used is not enabled,
    /// then this will panic with an `UnrecognizedFormat` error.
    pub bytes: Arc<[u8]>,
}

impl AudioSource {
    fn new(filename: &str) -> Self {
        let mut file = BufReader::new(File::open(filename).unwrap());

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        Self { bytes: bytes.into() }
    }
}

pub struct SoundSystem {
    audio_output: AudioOutput,
    bullet_sink: Sink,
    explosion_sink: Sink,
    player_shooting_source: AudioSource,
    enemy_destroyed_source: AudioSource,
}

impl SoundSystem {
    pub fn new() -> Self {
        let audio_output = AudioOutput::default();
        let bullet_sink = Sink::try_new(audio_output.stream_handle.as_ref().unwrap()).unwrap();
        let explosion_sink = Sink::try_new(audio_output.stream_handle.as_ref().unwrap()).unwrap();

        bullet_sink.set_speed(1.5);
        explosion_sink.set_speed(2.0);

        let player_shooting_source = AudioSource::new("assets/Audio/Player_SFX/player_shooting_one.wav");
        let enemy_destroyed_source = AudioSource::new("assets/Audio/Enemy_SFX/enemy_Spider_DestroyedExplosion.wav");

        Self {
            audio_output,
            bullet_sink,
            explosion_sink,
            player_shooting_source,
            enemy_destroyed_source,
        }
    }

    pub fn play_player_shooting(&self) {
        let data = self.player_shooting_source.bytes.clone();
        let source = Decoder::new(Cursor::new(data)).unwrap();
        self.bullet_sink.clear();
        self.bullet_sink.append(source);
        self.bullet_sink.play();
    }

    pub fn play_enemy_destroyed(&self) {
        let data = self.enemy_destroyed_source.bytes.clone();
        let source = Decoder::new(Cursor::new(data)).unwrap();
        self.explosion_sink.clear();
        self.explosion_sink.append(source);
        self.explosion_sink.play();
    }
}
