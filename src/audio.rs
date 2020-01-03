extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

pub const SAMPLE_RATE: usize = 44_100;

pub fn initialize(context: &sdl2::Sdl) -> Result<sdl2::audio::AudioQueue<f32>, String> {
    let audio_subsystem = context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(1), // mono
        samples: Some(735),     // default sample size
    };

    audio_subsystem.open_queue(None, &desired_spec)
}
