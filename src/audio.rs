extern crate sdl2;

use sdl2::audio::{AudioSpecDesired};

pub fn initialize(context: &sdl2::Sdl) -> Result<sdl2::audio::AudioQueue<f32>, String> {
    let audio_subsystem = context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    audio_subsystem.open_queue(None, &desired_spec)
}
